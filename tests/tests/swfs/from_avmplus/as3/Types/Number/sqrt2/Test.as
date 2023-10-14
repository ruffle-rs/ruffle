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


// var SECTION = "15.8.2.8";
// var VERSION = "AS3";
// var TITLE   = "public static const SQRT2:Number = 1.4142135623730951;";



var num_sqrt2:Number = 1.4142135623730951;

Assert.expectEq("Number.SQRT2", num_sqrt2, Number.SQRT2);
Assert.expectEq("typeof Number.SQRT2", "Number", getQualifiedClassName(Number.SQRT2));

Assert.expectEq("Number.SQRT2 - DontDelete", false, delete(Number.SQRT2));
Assert.expectEq("Number.SQRT2 is still ok", num_sqrt2, Number.SQRT2);

Assert.expectEq("Number.SQRT2 - DontEnum", '',getNumberProp('SQRT2'));
Assert.expectEq("Number.SQRT2 is no enumberable", false, Number.propertyIsEnumerable('SQRT2'));

Assert.expectError("Number.SQR_2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.SQRT2 = 0; });
Assert.expectEq("Number.SQRT2 is still here", num_sqrt2, Number.SQRT2);


