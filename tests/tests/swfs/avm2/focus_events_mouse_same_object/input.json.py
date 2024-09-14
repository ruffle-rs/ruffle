#!/usr/bin/env python3

coord_map = {
    'void':    [1,   1],
    'spriteA': [1,   101],
    'mc1A':    [101, 101],
    'mc2A':    [201, 101],
    'mc3A':    [301, 101],
    'textA':   [401, 101],
    'buttonA': [501, 101],
}

click_seq = [
    'spriteA',
    'spriteA',
    'esc',
    'spriteA',
    'spriteA',
    'esc',
]

print('[')

for obj in click_seq:
    if obj == 'esc':
        print(f'    {{ "type": "KeyDown", "key_code": 27 }},')
        print(f'    {{ "type": "KeyUp", "key_code": 27 }},')
        continue
    pos = coord_map[obj]
    print(f'    {{ "type": "MouseMove", "pos": {pos} }},')
    print(f'    {{ "type": "MouseDown", "pos": {pos}, "btn": "Left" }},')
    print(f'    {{ "type": "MouseUp", "pos": {pos}, "btn": "Left" }},')

print(f'    {{ "type": "Wait" }}')
print(']')
