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


// var SECTION = "15.8.1.6";
// var VERSION = "AS3";
// var TITLE   = "public static const PI:Number = 3.141592653589793;";



var num_pi:Number = 3.141592653589793;

Assert.expectEq("Number.PI", num_pi, Number.PI);
Assert.expectEq("typeof Number.PI", "Number", getQualifiedClassName(Number.PI));

Assert.expectEq("Number.PI - DontDelete", false, delete(Number.PI));
Assert.expectEq("Number.PI is still ok", num_pi, Number.PI);

Assert.expectEq("Number.PI - DontEnum", '',getNumberProp('PI'));
Assert.expectEq("Number.PI is no enumberable", false, Number.propertyIsEnumerable('PI'));

Assert.expectError("Number.PI - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.PI = 0; });
Assert.expectEq("Number.PI is still here", num_pi, Number.PI);


