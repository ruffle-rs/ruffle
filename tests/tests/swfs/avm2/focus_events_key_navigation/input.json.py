#!/usr/bin/env python3

key_seq = [
    'tab',
    'tab',
    'right',
    'down',
    'down',
    'left',
    'left',
    'up',
    'left',
    'down',
    'down',
    'right',
    'down',
    'esc',
]

print('[')

for obj in key_seq:
    key = None
    if obj == 'esc':
        key = 27
    elif obj == 'tab':
        key = 9
    elif obj == 'left':
        key = 37
    elif obj == 'up':
        key = 38
    elif obj == 'right':
        key = 39
    elif obj == 'down':
        key = 40
    print(f'    {{ "type": "KeyDown", "key_code": {key} }},')
    print(f'    {{ "type": "KeyUp", "key_code": {key} }},')

print(f'    {{ "type": "Wait" }}')
print(']')
