// WCAG relative luminance, measured from production Rust color constants.
import fs from 'node:fs';
const widgets=fs.readFileSync('crates/aether_view/src/widgets.rs','utf8');
const color=name=>{const m=widgets.match(new RegExp(`pub const ${name}: Color = Color::srgb\\(([^)]+)\\)`));if(!m)throw Error('Missing '+name);return m[1].split(',').map(Number);};
const luminance=c=>c.map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4).reduce((s,v,i)=>s+v*[.2126,.7152,.0722][i],0);
const contrast=(a,b)=>{const x=luminance(a),y=luminance(b);return (Math.max(x,y)+.05)/(Math.min(x,y)+.05);};
const background=[.035,.050,.075];
const worst=background.map(v=>v*.90+.10); // translucent panel over pure white, conservative.
const entries=[
  ['Texte principal / panneau',color('PAPER'),color('INK')],
  ['Texte secondaire / panneau translucide, fond blanc',color('MUTED'),worst],
  ['Bouton principal et appuyé',color('INK'),color('ACCENT')],
  ['Bouton secondaire',color('PAPER'),[.10,.135,.18]],
  ['Survol principal',color('INK'),[.77,.65,1]],
  ['Survol secondaire',color('PAPER'),[.26,.24,.39]],
  ['Focus clavier',color('PAPER'),[.26,.24,.39]],
];
const report=entries.map(([state,foreground,background])=>({state,foreground,background,ratio:contrast(foreground,background),minimum:4.5}));
fs.writeFileSync('docs/evidence/contrast.json',JSON.stringify({method:'WCAG sRGB relative luminance; same ratios in luminance grayscale',states:report,pass:report.every(v=>v.ratio>=v.minimum)},null,2));
for(const r of report)console.log(`${r.state}: ${r.ratio.toFixed(2)}:1`);
if(report.some(v=>v.ratio<v.minimum))process.exitCode=1;
