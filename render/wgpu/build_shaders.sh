#!/bin/sh
glslc ./shaders/color.vert -o ./shaders/color.vert.spv
glslc ./shaders/texture.vert -o ./shaders/texture.vert.spv

glslc ./shaders/color.frag -o ./shaders/color.frag.spv
glslc -DSRGB_RENDER_TARGET ./shaders/color.frag -o ./shaders/color_srgb.frag.spv
glslc ./shaders/bitmap.frag -o ./shaders/bitmap.frag.spv
glslc -DSRGB_RENDER_TARGET ./shaders/bitmap.frag -o ./shaders/bitmap_srgb.frag.spv
glslc ./shaders/gradient.frag -o ./shaders/gradient.frag.spv
glslc -DSRGB_RENDER_TARGET ./shaders/gradient.frag -o ./shaders/gradient_srgb.frag.spv
