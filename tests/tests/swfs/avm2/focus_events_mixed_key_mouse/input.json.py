#!/usr/bin/env python3

coord_map = {
    'void':    [1,   1],
    'sprite1': [1,   101],
    'sprite2': [101, 101],
    'sprite3': [201, 101],
}

click_seq = [
    'tab', 'tab', 'esc',
    'sprite1', 'tab', 'esc',
    'sprite2', 'tab', 'esc',
    'tab', 'sprite2', 'esc',
    'sprite3', 'tab', 'esc',
    'void', 'esc',
]

print('[')

for obj in click_seq:
    if obj == 'esc':
        print(f'    {{ "type": "KeyDown", "key": "Escape" }},')
        print(f'    {{ "type": "KeyUp", "key": "Escape" }},')
        continue
    if obj == 'tab':
        print(f'    {{ "type": "KeyDown", "key": "Tab" }},')
        print(f'    {{ "type": "KeyUp", "key": "Tab" }},')
        continue
    pos = coord_map[obj]
    print(f'    {{ "type": "MouseMove", "pos": {pos} }},')
    print(f'    {{ "type": "MouseDown", "pos": {pos}, "btn": "Left" }},')
    print(f'    {{ "type": "MouseUp", "pos": {pos}, "btn": "Left" }},')

print(f'    {{ "type": "FocusLost" }}')
print(']')
