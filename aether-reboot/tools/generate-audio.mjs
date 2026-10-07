// Original procedural cues; no external sound recording. 16-bit mono PCM, 44.1 kHz.
import fs from 'node:fs';
fs.mkdirSync('assets/audio',{recursive:true});
for(const [name,duration,frequencies] of [['click',0.075,[640,960]],['bell',0.55,[523.25,659.25,783.99]],['warning',0.25,[220,277]],['tension',0.18,[110,165]]]) {
  const count=Math.floor(duration*44100), b=Buffer.alloc(44+count*2);
  b.write('RIFF');b.writeUInt32LE(b.length-8,4);b.write('WAVEfmt ',8);b.writeUInt32LE(16,16);b.writeUInt16LE(1,20);b.writeUInt16LE(1,22);b.writeUInt32LE(44100,24);b.writeUInt32LE(88200,28);b.writeUInt16LE(2,32);b.writeUInt16LE(16,34);b.write('data',36);b.writeUInt32LE(count*2,40);
  for(let i=0;i<count;i++){const t=i/44100,envelope=Math.min(t/0.008,1)*Math.exp(-t/duration*6)*(1-i/count);const v=frequencies.reduce((s,f)=>s+Math.sin(t*2*Math.PI*f),0)/frequencies.length;b.writeInt16LE(Math.round(v*envelope*20000),44+i*2);}
  fs.writeFileSync(`assets/audio/${name}.wav`,b);
}
