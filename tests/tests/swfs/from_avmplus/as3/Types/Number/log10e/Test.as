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


// var SECTION = "15.8.1.5";
// var VERSION = "AS3";
// var TITLE   = "public static const LOG10E:Number = 0.4342944819032518";



var num_log10e:Number = 0.4342944819032518;

Assert.expectEq("Number.LOG10E", num_log10e, Number.LOG10E);
Assert.expectEq("typeof Number.LOG10E", "Number", getQualifiedClassName(Number.LOG10E));

Assert.expectEq("Number.LOG10E - DontDelete", false, delete(Number.LOG10E));
Assert.expectEq("Number.LOG10E is still ok", num_log10e, Number.LOG10E);

Assert.expectEq("Number.LOG10E - DontEnum", '',getNumberProp('LOG10E'));
Assert.expectEq("Number.LOG10E is no enumberable", false, Number.propertyIsEnumerable('LOG10E'));

Assert.expectError("Number.LOG10E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LOG10E = 0; });
Assert.expectEq("Number.LOG10E is still here", num_log10e, Number.LOG10E);

// NOTE The value of Math.LOG10E is approximately the reciprocal of the value of Math.LN10.
Assert.expectEq("Number.LOG10E is approximately the reciprocal of the value of Number.LN10", 1/Number.LN10, Number.LOG10E);


