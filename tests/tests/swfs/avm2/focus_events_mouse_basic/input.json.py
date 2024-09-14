#!/usr/bin/env python3

coord_map = {
    'void':    [1,   1],
    'sprite1': [1,   101],
    'sprite2': [101, 101],
    'mc1':     [201, 101],
    'mc2':     [301, 101],
    'text':    [401, 101],
    'button1': [501, 101],
    'button2': [601, 101],
}

click_seq = [
    'void', 'esc', # ================
    'sprite1', 'esc',
    'void', 'esc',
    'sprite2', 'esc',
    'void', 'esc',
    'mc1', 'esc',
    'void', 'esc',
    'mc2', 'esc',
    'void', 'esc',
    'text', 'esc',
    'void', 'esc',
    'button1', 'esc',
    'void', 'esc',
    'button2', 'esc',
    'void', 'esc', # ================
    'sprite1', 'esc',
    'sprite2', 'esc',
    'mc1', 'esc',
    'mc2', 'esc',
    'text', 'esc',
    'button1', 'esc',
    'button2', 'esc',
    'void', 'esc', # ================
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
