"""Verify license inventory against actual distribution bytes."""
from pathlib import Path
import argparse,hashlib,json,zipfile

def verify(root):
 index=json.loads((root/'third_party/dependency-licenses/index.json').read_text('utf-8-sig'))
 failures=[];count=0
 for package in index['packages']:
  for notice in package['notices']:
   count+=1;p=root/'third_party/dependency-licenses'/notice['file']
   if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=notice['sha256']: failures.append(notice['file'])
 if failures: raise SystemExit(f'License inventory mismatch: {len(failures)} of {count} files')
 print(f'PASS: {count} license files exactly match inventory SHA-256.')

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1]);verify(parser.parse_args().root)
