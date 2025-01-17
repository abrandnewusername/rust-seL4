import sys
import argparse
import pexpect
from pathlib import Path

TIMEOUT = 60 * 10

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('dir', type=Path)
    args = parser.parse_args()
    run(args)

def run(args):
    child = pexpect.spawn(str(args.dir / 'simulate'), cwd=args.dir, encoding='utf-8')
    child.logfile = sys.stdout
    ix = child.expect(['All is well in the universe', 'halting...', pexpect.TIMEOUT], timeout=TIMEOUT)
    print()
    if ix != 0:
        if ix == 1:
            sys.exit('> test reported failure')
        if ix == 2:
            sys.exit('> test timed out')
        assert False

if __name__ == '__main__':
    main()
