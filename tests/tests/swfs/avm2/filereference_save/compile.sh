#!/bin/bash
docker run -it --rm -v ${PWD}:/app instrumentisto/flex-sdk mxmlc -output test.swf -compiler.debug=true Test.as
