#!/usr/bin/python3
"""Prepare a self-contained RPM source archive; network is only used here."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tarfile,tempfile,urllib.request
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'_rpm'
name='mission-center-custom-0.6.1'
for directory in ('SOURCES','SPECS','BUILD','BUILDROOT','RPMS','SRPMS'):
 (OUT/directory).mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='prepare-',dir=OUT) as temporary:
 source=Path(temporary)/name
 shutil.copytree(ROOT,source,ignore=shutil.ignore_patterns('.git','_build','_rpm','target','__pycache__'))
 env=dict(os.environ,CARGO_HOME=str(ROOT/'_build/cargo-home'))
 vendor=subprocess.check_output(['cargo','vendor','--locked','--versioned-dirs','--sync','src/sys_info_v2/gatherer/Cargo.toml','vendor'],cwd=source,env=env,text=True)
 (source/'.cargo').mkdir(exist_ok=True)
 (source/'.cargo/config.toml').write_text(vendor+'\n[net]\noffline = true\n')
 cache=source/'build-sources';cache.mkdir()
 for manifest in (source/'src/sys_info_v2/gatherer/3rdparty').glob('*/*.json'):
  package=json.loads(manifest.read_text())
  url=package['source-url'];filename=url.rsplit('/',1)[-1]
  path=cache/filename
  print('Downloading pinned native source:',url,flush=True)
  urllib.request.urlretrieve(url,path)
  assert hashlib.sha256(path.read_bytes()).hexdigest()==package['source-hash'],f'Invalid checksum: {filename}'
 archive=OUT/'SOURCES'/f'{name}.tar.xz'
 with tarfile.open(archive,'w:xz') as output:
  output.add(source,arcname=name)
 shutil.copy2(ROOT/'packaging/mission-center-custom.spec',OUT/'SPECS')
 print(archive,flush=True)
