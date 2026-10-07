"""R57 freshly recomputed matched CPU volumetric inspection, not an engine screenshot or a paintover.

Both fields use identical rays, phase, absorption and fixed erosion values.
Live noise, midpoint jitter, orthographic rays and display transform retain R47 controls.
R55 adds actual low sun, exact offset shadow taps and the directional bounce.
Both historical four-step and current two-step lighting refresh are tested.
No opaque scene depth, temporal reprojection, live advection or post effects are modeled.
"""
import bpy,numpy as np,json,struct,time,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OLD=ROOT/'tools/art/cloud-r47/archive/baseline'
OUT=ROOT/'tools/art/cloud-r57/review';OUT.mkdir(parents=True,exist_ok=True)
WIDTH,HEIGHT=800,500
BANK_INDEX=0
ORIGIN=[3250,2200,3600]
TARGET=[0,-90,0]
SPAN=6200
SUN=np.array([-.85356265,-.050973576,-.5184906],dtype=np.float32);SUN/=np.linalg.norm(SUN)
LIGHTING_REFRESH=4
horizontal=SUN*np.array([1,0,1],dtype=np.float32);horizontal/=np.linalg.norm(horizontal)
BOUNCE=horizontal*.7431448255+np.array([0,.6691306064,0],dtype=np.float32)

def phase(cos_angle):
    g=.45
    return np.clip((1-g*g)/np.maximum(.1,1+g*g-2*g*cos_angle)**1.5,.65,1.7)


def smooth(a,b,x):
    t=np.clip((x-a)/(b-a),0,1);return t*t*(3-2*t)

def load(root):
    bank=json.loads((root/'assets/atmosphere/cloud-banks.json').read_text())[BANK_INDEX]
    raw=(root/'assets/atmosphere'/str(bank['key']+'-shape.ktx2')).read_bytes()
    nx,ny,nz=struct.unpack_from('<3I',raw,20);offset,length,_=struct.unpack_from('<3Q',raw,80)
    cache=np.frombuffer(raw,dtype=np.uint8,count=length,offset=offset).reshape((nz,ny,nx,4))
    return bank,cache

def inspect(root,label,mode='cache'):
    started=time.perf_counter();bank,data=load(root)
    center=np.array(bank['center'],dtype=np.float32);extent=np.array(bank['extent'],dtype=np.float32)
    size=extent+2*np.array(bank['padding'],dtype=np.float32);dims=np.array(bank['resolution'])
    def sample(points):
        pos=(points/size+.5)*dims-.5;lo=np.floor(pos).astype(np.int32);f=(pos-lo).astype(np.float32)
        out=np.zeros((len(points),4),dtype=np.float32)
        for z in range(2):
            for y in range(2):
                for x in range(2):
                    bit=np.array([x,y,z]);ix=np.clip(lo+bit,0,dims-1)
                    w=np.prod(np.where(bit,f,1-f),axis=-1)
                    out+=data[ix[:,2],ix[:,1],ix[:,0]]*(w[:,None]/255)
        return out
    def density(points):
        h=points[:,1]/extent[1]+.5
        radial=np.linalg.norm(points[:,(0,2)]/(extent[(0,2),]*.5),axis=-1)
        if mode=='analytic':
            from build_cloud_cache import union
            field=union(points+center,bank)
            field[:,1:]=field[:,1:]*.5+.5
        else:
            field=sample(points)
            if mode=='cache-density-analytic-normal':
                from build_cloud_cache import union
                field[:,1:]=union(points+center,bank)[:,1:]*.5+.5
        floor=.22*smooth(.025,.09,h)*(1-smooth(.26,.43,h))
        shaped=smooth(bank['shape'][0]-.60,.35,field[:,0]-.24-.098)
        d=np.maximum(shaped,floor*(.75+.70*.25))*(1-smooth(.80,.99,radial))*smooth(.01,.08,h)*(1-smooth(.90,.99,h))
        d=np.where((h>.01)&(h<.99)&(radial<1.05),d,0)
        normal=field[:,1:]*2-1;normal/=np.maximum(np.linalg.norm(normal,axis=-1,keepdims=True),1e-8)
        return d,normal
    # Oblique view sees upper folds, side cavities, and the unchanged base.
    origin=np.array(ORIGIN,dtype=np.float32);target=np.array(TARGET,dtype=np.float32)
    forward=target-origin;forward/=np.linalg.norm(forward)
    right=np.cross(forward,np.array([0,1,0],dtype=np.float32));right/=np.linalg.norm(right)
    up=np.cross(right,forward)
    xx,yy=np.meshgrid((np.arange(WIDTH)+.5)/WIDTH-.5,.5-(np.arange(HEIGHT)+.5)/HEIGHT)
    origins=origin+(xx[...,None]*right+yy[...,None]*up*HEIGHT/WIDTH)*SPAN
    origins=origins.reshape((-1,3)).astype(np.float32)
    rays=np.broadcast_to(forward,origins.shape)
    a=(-extent*.5-origins)/rays;b=(extent*.5-origins)/rays
    near=np.max(np.minimum(a,b),axis=1);far=np.min(np.maximum(a,b),axis=1)
    valid=(far>np.maximum(near,0));near=np.maximum(near,0)
    step=(far-near)/128;trans=np.ones(len(origins),dtype=np.float32);light=np.zeros_like(origins)
    shade=np.zeros_like(origins)
    direct_phase=phase(float(forward@SUN))
    # These remain the historical orthographic rays. Their incoming view
    # direction is constant; production perspective rays vary per pixel.
    bounce_phase=phase(float(forward@BOUNCE))
    def transmission(q,direction):
        optical=np.zeros(len(q),dtype=np.float32);distance=18.;stride=30.
        side=np.cross(direction,np.array([0,0,1],dtype=np.float32));side/=np.linalg.norm(side)
        for j in range(4):
            offset=side*(1 if j%2==0 else -1)*distance*.10
            optical+=density(q+direction*distance+offset)[0]*stride
            distance+=stride;stride*=1.9
        extinction=optical*bank['shape'][1]
        return np.exp(-extinction)*.8+np.exp(-extinction*.27)*.2
    for k in range(128):
        ids=np.flatnonzero(valid&(trans>.012))
        p=origins[ids]+rays[ids]*(near[ids]+(k+.5)*step[ids])[:,None]
        d,normal=density(p)
        dense=d>.001;active=ids[dense];p=p[dense];normal=normal[dense];d=d[dense]
        if not len(active):continue
        update=(k%LIGHTING_REFRESH==0)|(trans[active]>.99)
        if np.any(update):
            q=p[update];scattering=transmission(q,SUN)
            powder=.65+.35*(1-np.exp(-d[update]*4))
            h=np.clip(q[:,1]/extent[1]+.5,0,1)
            exposure=.2+.8*np.maximum(0,normal[update]@SUN)
            ambient=(np.array([.045,.10,.22])*(1-h[:,None])+np.array([.18,.30,.53])*h[:,None])*.65*np.array([4.,4.5,5.])
            direct=np.array([3.,1.86,.96])*scattering[:,None]*powder[:,None]*direct_phase*exposure[:,None]
            bounce=np.array([3.,1.86,.96])*.35*transmission(q,BOUNCE)[:,None]*powder[:,None]*bounce_phase
            shade[active[update]]=ambient+direct+bounce
        alpha=1-np.exp(-d*step[active]*bank['shape'][1])
        light[active]+=shade[active]*alpha[:,None]*trans[active,None];trans[active]*=1-alpha
    background=np.array([.14,.23,.36],dtype=np.float32)
    linear=light+trans[:,None]*background
    rgb=np.power(np.clip(linear/(1+linear),0,1),1/2.2)
    pixels=np.concatenate((rgb,np.ones((len(rgb),1),dtype=np.float32)),axis=1).reshape((HEIGHT,WIDTH,4))
    image=bpy.data.images.new('Native CPU cloud field '+label,width=WIDTH,height=HEIGHT,alpha=True,float_buffer=True)
    image.pixels.foreach_set(pixels[::-1].flatten());image.filepath_raw=str(OUT/('cloud-field-'+label+'.png'));image.file_format='PNG';image.save()
    info={'label':label,'bank':bank['key'],'mode':mode,'seconds':time.perf_counter()-started,'dimensions':[WIDTH,HEIGHT],'origin_local':ORIGIN,'target_local':TARGET,'orthographic_span':SPAN,'steps':128,'lighting_refresh_steps':LIGHTING_REFRESH,'lighting_recipe':'r57-blue-fill-warm-direct','sun_direction':SUN.tolist(),'bounce_direction':BOUNCE.tolist(),'shadow_taps_per_lobe':4,'daylight_factor':1,'live_noise':'held at R=.70,G/B=.65,A=.50 for both fields','alpha_mean':float(np.mean(1-trans)),'note':'CPU field inspection, not a runtime screenshot'}
    (OUT/('cloud-field-'+label+'.json')).write_text(json.dumps(info,indent=2)+'\n')
    print(json.dumps(info),flush=True)
    return {'linear':linear,'display':rgb,'alpha':1-trans,'metadata':info}

if __name__=='__main__':
    sys.path.insert(0,str(Path(__file__).resolve().parent))
    selected=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else ['before','after']
    for label in selected:inspect(OLD if label=='before' else ROOT,label)
