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


// var SECTION = "15.8.1.2";
// var VERSION = "AS3";
// var TITLE   = "public static const LN10:Number = 2.302585092994046;";



var num_ln10:Number = 2.302585092994046;

Assert.expectEq("Number.LN10", num_ln10, Number.LN10);
Assert.expectEq("typeof Number.LN10", "Number", getQualifiedClassName(Number.LN10));

Assert.expectEq("Number.LN10 - DontDelete", false, delete(Number.LN10));
Assert.expectEq("Number.LN10 is still ok", num_ln10, Number.LN10);

Assert.expectEq("Number.LN10 - DontEnum", '',getNumberProp('LN10'));
Assert.expectEq("Number.LN10 is no enumberable", false, Number.propertyIsEnumerable('LN10'));

Assert.expectError("Number.LN10 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LN10 = 0; });
Assert.expectEq("Number.LN10 is still here", num_ln10, Number.LN10);


