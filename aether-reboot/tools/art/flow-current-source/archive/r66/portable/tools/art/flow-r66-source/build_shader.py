"""Build exactly one R66 optical-profile delta, no other shader parameter."""
from pathlib import Path
import hashlib
A=Path(__file__).resolve().parent;R=A.parents[2]
BEFORE='let optical_profile=.12+1.88*middle;';AFTER='let optical_profile=1.0;'
raw=(A/'history/world-flow-r64.wgsl').read_bytes()
assert hashlib.sha256(raw).hexdigest()=='0861bdc4197dab68fb10826a6165fa6ef5e325fa52ad6d42331a6b65e30c5cf5'
assert raw.count(BEFORE.encode())==1
out=raw.replace(BEFORE.encode(),AFTER.encode())
(R/'assets/shaders/world-flow.wgsl').write_bytes(out)
print('R66 shader',hashlib.sha256(out).hexdigest())
