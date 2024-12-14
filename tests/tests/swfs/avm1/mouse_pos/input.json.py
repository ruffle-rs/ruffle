# You can use this to recreate input.json from Flash Player output if necessary.

import re

first = True
with open('output.txt', 'rt') as f:
    print('[')
    for line in f:
        if m := re.match('^_root (\d+) (\d+)', line):
            x, y = m.groups()
            if not first:
                print(',')
            print(f'  {{"type": "MouseMove", "pos": [{x}, {y}]}}, {{"type": "Wait"}}', end='')
            first = False
    print('\n]')
