#!/usr/bin/env python3

import sys
from os import path

sys.stdout.reconfigure(encoding='utf-8')

input_file = path.join(path.dirname(__file__), 'input.txt')
with open(input_file, 'r', encoding='utf-8') as file:
    characters = file.read().replace(' ', '').replace('\n', '')

print('[')
for ch in characters:
    print(f'    {{ "type": "TextInput", "codepoint": "{ch}" }},')

print(f'    {{ "type": "KeyDown", "key_code": 27 }}')

print(']')
