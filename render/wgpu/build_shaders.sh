#!/bin/sh
glslangValidator -V ./shaders/color.frag -o ./shaders/color.frag.spv
glslangValidator -V ./shaders/color.vert -o ./shaders/color.vert.spv
glslangValidator -V ./shaders/bitmap.frag -o ./shaders/bitmap.frag.spv
glslangValidator -V ./shaders/gradient.frag -o ./shaders/gradient.frag.spv
glslangValidator -V ./shaders/texture.vert -o ./shaders/texture.vert.spv
