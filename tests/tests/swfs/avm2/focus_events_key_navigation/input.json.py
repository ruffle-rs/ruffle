#!/usr/bin/env python3

key_seq = [
    'Tab',
    'Tab',
    'ArrowRight',
    'ArrowDown',
    'ArrowDown',
    'ArrowLeft',
    'ArrowLeft',
    'ArrowUp',
    'ArrowLeft',
    'ArrowDown',
    'ArrowDown',
    'ArrowRight',
    'ArrowDown',
    'Escape',
]

print('[')

for key in key_seq:
    print(f'    {{ "type": "KeyDown", "key": "{key}" }},')
    print(f'    {{ "type": "KeyUp", "key": "{key}" }},')

print(f'    {{ "type": "Wait" }}')
print(']')
