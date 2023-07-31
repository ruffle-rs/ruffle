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


// var SECTION = "15.8.1.4";
// var VERSION = "AS3";
// var TITLE   = "public static const LOG2E:Number = 1.442695040888963387;";



var num_log2e:Number = 1.442695040888963387;

Assert.expectEq("Number.LOG2E", num_log2e, Number.LOG2E);
Assert.expectEq("typeof Number.LOG2E", "Number", getQualifiedClassName(Number.LOG2E));

Assert.expectEq("Number.LOG2E - DontDelete", false, delete(Number.LOG2E));
Assert.expectEq("Number.LOG2E is still ok", num_log2e, Number.LOG2E);

Assert.expectEq("Number.LOG2E - DontEnum", '',getNumberProp('LOG2E'));
Assert.expectEq("Number.LOG2E is no enumberable", false, Number.propertyIsEnumerable('LOG2E'));

Assert.expectError("Number.LOG2E - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LOG2E = 0; });
Assert.expectEq("Number.LOG2E is still here", num_log2e, Number.LOG2E);

// NOTE The value of Math.LOG2E is approximately the reciprocal of the value of Math.LN2.
Assert.expectEq("Number.LOG2E is approximately the reciprocal of the value of Number.LN2", 1/Number.LN2, Number.LOG2E);


