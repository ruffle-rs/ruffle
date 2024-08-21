#!/bin/bash
docker run -it --rm -v ${PWD}:/app instrumentisto/flex-sdk mxmlc -swf-version 13 -output test.swf -compiler.debug=true Test.as
