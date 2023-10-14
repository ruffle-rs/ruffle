All files here are copied and adapted from Adobe's [avmplus repo](https://github.com/adobe/avmplus/tree/65a05927767f3735db37823eebf7d743531f5d37/test/acceptance).

# License
The original tests are licensed [MPL 2.0](http://mozilla.org/MPL/2.0/), and therefore these adaptions are too.

# Adapting a new test
To adapt a new test:
- Take the AS file from the above repo, and put it in an appropriate folder somewhere here.
- If the test is conditional on, for example, `swfVersion()` - separate it manually into multiple tests.
- By hand, remove any references to `avmplus.System` or similar classes - they don't exist here.
- Create an mxmlc config and ensure that `lib` is added to the source paths.

You may wish to set the movie class to `Test` and have the script reside at `Test.as`, and then add the following to the top:
```actionscript
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}
```

# Compiling
Unless there's a fla, all tests are compiled with mxmlc. To compile, use `mxmlc -load-config+=config.xml Test.as`

If you need to make a fla then:
- Ensure that the `lib` folder is added to the source path of the movie, so it can pick up Assert/Util.
- If required, disable Strict Mode in the AS3 settings to make some tests compile.
