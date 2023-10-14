/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
import com.adobe.test.Assert;
import com.adobe.test.Utils;

import flash.utils.getQualifiedClassName;

//var dummy_number = NaN;
//var isAS3:Boolean = dummy_number.toString == dummy_number.AS3::toString;

function getNumberProp(name):String
{
  string = '';
  for ( prop in Number )
  {
    string += ( prop == name ) ? prop : '';
  }
  return string;
}


// var SECTION = "15.8.1.7";
// var VERSION = "AS3";
// var TITLE   = "public static const SQRT1_2:Number = 0.7071067811865476;";



var num_sqrt1_2:Number = 0.7071067811865476;

Assert.expectEq("Number.SQRT1_2", num_sqrt1_2, Number.SQRT1_2);
Assert.expectEq("typeof Number.SQRT1_2", "Number", getQualifiedClassName(Number.SQRT1_2));

Assert.expectEq("Number.SQRT1_2 - DontDelete", false, delete(Number.SQRT1_2));
Assert.expectEq("Number.SQRT1_2 is still ok", num_sqrt1_2, Number.SQRT1_2);

Assert.expectEq("Number.SQRT1_2 - DontEnum", '',getNumberProp('SQRT1_2'));
Assert.expectEq("Number.SQRT1_2 is no enumberable", false, Number.propertyIsEnumerable('SQRT1_2'));

Assert.expectError("Number.SQRT1_2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.SQRT1_2 = 0; });
Assert.expectEq("Number.SQRT1_2 is still here", num_sqrt1_2, Number.SQRT1_2);

// NOTE The value of Math.SQRT1_2 is approximately the reciprocal of the value of Math.SQRT2.
Assert.expectEq("Number.SQRT1_2 is approximately the reciprocal of the value of Number.SQRT2", 1/Number.SQRT2, Number.SQRT1_2);


