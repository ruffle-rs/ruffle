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

/*
The Number value for e, the base of the natural logarithms, which is approximately
2.7182818284590452354. This property has the attributes { [[Writable]]: false,
[[Enumerable]]: false, [[Configurable]]: false }.
*/

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

// var SECTION = "15.8.1.1";
// var VERSION = "AS3";
// var TITLE   = "public static const E:Number = 2.718281828459045;";


var num_e:Number = 2.718281828459045;

Assert.expectEq("Number.E", num_e, Number.E);
Assert.expectEq("typeof Number.E", "Number", getQualifiedClassName(Number.E));

Assert.expectEq("Number.E - DontDelete", false, delete(Number.E));
Assert.expectEq("Number.E is still ok", num_e, Number.E);

Assert.expectEq("Number.E - DontEnum", '', getNumberProp('E'));
Assert.expectEq("Number.E is no enumberable", false, Number.propertyIsEnumerable('E'));

Assert.expectError("Number.E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.E = 0; });
Assert.expectEq("Number.E is still here", num_e, Number.E);


