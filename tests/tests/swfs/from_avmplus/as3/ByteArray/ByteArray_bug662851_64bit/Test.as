/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip;public class Test extends MovieClip {}}

    import flash.utils.ByteArray
    import avmplus.* ;      // System class in the avmshell
import com.adobe.test.Assert;
import com.adobe.test.Utils;

//     var SECTION = "ByteArray";
//     var VERSION = "as3";
//     var TITLE   = "test ByteArray class exceeding MMgc::GCHeap::kMaxObjectSize";


    /* bz: Bug 662851 - Make EnsureWritableCapacity take uint32_t instead of uint64_t since all callers pass uint32_t
       First 2 testcases will produce an error via mmfx_new_array_opt() returning null WHEN  run on 32bit platform,
       however on 64bit platforms this will succeed to allocate a lot of memory, so do not run these on 64bit
           http://hg.mozilla.org/tamarin-redux/annotate/47d6d75afd61/core/ByteArrayGlue.cpp#l147
       Last 2 testcases will produce an error via the minimumCapacity check
           http://hg.mozilla.org/tamarin-redux/annotate/47d6d75afd61/core/ByteArrayGlue.cpp#l132
    */

    var is32bit:Boolean = false;
    var i:Number;
    if (is32bit) {
	i=0xFFFFC000;
    } else {
	i=0xFFFFE000;
    }

    for(;i<0x100000000;i+=4096)
    {
	var expected:String = "Error #1000";
        var result:String = "no error";
	try {
	    new ByteArray().length = i;
	} catch(err) {
	    result = Utils.grabError(err, err.toString());
	}
	Assert.expectEq("ByteArray.length "+ i, expected, result);
    }



