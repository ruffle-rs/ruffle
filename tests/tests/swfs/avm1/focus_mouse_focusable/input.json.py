#!/usr/bin/env python3

print('[')

for i in range(40):
    x = i * 100
    y = 100
    while x >= 1000:
        x -= 1000
        y += 100
    pos = [x + 10, y + 10]
    print(f'    {{ "type": "MouseMove", "pos": {pos} }},')
    print(f'    {{ "type": "MouseDown", "pos": {pos}, "btn": "Left" }},')
    print(f'    {{ "type": "MouseUp", "pos": {pos}, "btn": "Left" }},')

pos = [10, 10]
print(f'    {{ "type": "MouseMove", "pos": {pos} }},')
print(f'    {{ "type": "MouseDown", "pos": {pos}, "btn": "Left" }},')
print(f'    {{ "type": "MouseUp", "pos": {pos}, "btn": "Left" }},')

print(f'    {{ "type": "Wait" }}')
print(']')
