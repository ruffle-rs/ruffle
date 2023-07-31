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


// var SECTION = "15.8.1.3";
// var VERSION = "AS3";
// var TITLE   = "public static const LN2:Number = 0.6931471805599453;";



var num_ln2:Number = 0.6931471805599453;

Assert.expectEq("Number.LN2", num_ln2, Number.LN2);
Assert.expectEq("typeof Number.LN2", "Number", getQualifiedClassName(Number.LN2));

Assert.expectEq("Number.LN2 - DontDelete", false, delete(Number.LN2));
Assert.expectEq("Number.LN2 is still ok", num_ln2, Number.LN2);

Assert.expectEq("Number.LN2 - DontEnum", '',getNumberProp('LN2'));
Assert.expectEq("Number.LN2 is no enumberable", false, Number.propertyIsEnumerable('LN2'));

Assert.expectError("Number.LN2 - ReadOnly", Utils.REFERENCEERROR+1074, function(){ Number.LN2 = 0; });
Assert.expectEq("Number.LN2 is still here", num_ln2, Number.LN2);


