/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Vector";
// var VERSION = "Bug 592735: code coverage";


function g(x) : uint { return x }


var v:Vector.<uint> = new Vector.<uint>;
var x:uint;
x = 20;
// avmplus::NativeID::__AS3___vec_Vector_uint_AS3_push_thunk(MethodEnv*,uint32_t,Atom*)
v.push(g(x)/10);
Assert.expectEq("Vector_uint_AS3_push", 1, v.length);
v.push();
Assert.expectEq("Vector_uint_AS3_push null", 1, v.length);

// avmplus::NativeID::__AS3___vec_Vector_uint_AS3_unshift_thunk(MethodEnv*,uint32_t,Atom*)
x = 10;
v.unshift(g(x)/10);
Assert.expectEq("Vector_uint_AS3_unshift", "1,2", v.toString());
v.unshift();
Assert.expectEq("Vector_uint_AS3_unshift null", "1,2", v.toString());

// avmplus::NativeID::__AS3___vec_Vector_uint_AS3_shift_thunk(MethodEnv*,uint32_t,Atom*)
Assert.expectEq("Vector_uint_AS3_shift", 1, v.shift());
Assert.expectEq("Vector_uint_AS3_shift length", 1, v.length);

// avmplus::NativeID::__AS3___vec_Vector_uint_AS3_pop_thunk(MethodEnv*,uint32_t,Atom*)
v.pop();
Assert.expectEq("Vector_uint_AS3_pop", 0, v.length);

// avmplus::NativeID::__AS3___vec_Vector_uint_length_set_thunk(MethodEnv*,uint32_t,Atom*)
v.length = 2;
Assert.expectEq("Vector_uint_length_set", 2, v.length);

// avmplus::NativeID::__AS3___vec_Vector_uint_fixed_set_thunk(MethodEnv*,uint32_t,Atom*)
// avmplus::NativeID::__AS3___vec_Vector_uint_fixed_get_thunk(MethodEnv*,uint32_t,Atom*)
v.fixed = true;
Assert.expectEq("Vector_uint_fixed_get", true, v.fixed);

var expected = "RangeError: Error #1126";
var err = "exception not thrown";
try {
    v.push(g(x)/10);
}
catch (e:Error){
    err = e.toString();
}
Assert.expectEq("RangeError: Error #1126: Cannot change the length of a fixed Vector",
  expected,
  Utils.parseError(err, expected.length));

v.fixed = false;

// __AS3___vec_Vector_uint_private__map_thunk(MethodEnv*,uint32_t,Atom*)
x = 10;
v[0] = (g(x)/10);
x = 20;
v[1] = (g(x)/10);
function mapper1(value, index, obj)
{
    return value+1;
}
Assert.expectEq("Vector_uint_private__map", "2,3", v.map(mapper1).toString()); // NOTE: this returns a new vector and does not alter

// __AS3___vec_Vector_uint_private__reverse_thunk(MethodEnv*,uint32_t,Atom*)
Assert.expectEq("_Vector_uint_private__reverse", "2,1", v.reverse().toString());
