// Original source: https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/test/acceptance/Utils.as

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.adobe.test
{
import com.adobe.test.Assert;
public class Utils
{
  public static var GLOBAL = "[object global]";
  public static var PASSED = " PASSED!"
  public static var FAILED = " FAILED! expected: ";
  public static var PACKAGELIST = "{public,$1$private}::";
  public static var ARGUMENTERROR = "ArgumentError: Error #";
  public static var TYPEERROR = "TypeError: Error #";
  public static var REFERENCEERROR = "ReferenceError: Error #";
  public static var RANGEERROR = "RangeError: Error #";
  public static var URIERROR = "URIError: Error #";
  public static var EVALERROR = "EvalError: Error #";
  public static var VERIFYERROR = "VerifyError: Error #";

  // Return the "Error #XXXX" String from the error
  public static function grabError(err:*, str:String):String
  {
    var typeIndex:int = str.indexOf("Error:");
    var type:String = str.substr(0, typeIndex + 5);
    if (type == "TypeError") {
      Assert.expectEq("Asserting for TypeError", true, (err is TypeError));
    } else if (type == "ArgumentError") {
      Assert.expectEq("Asserting for ArgumentError", true, (err is ArgumentError));
    }
    var numIndex:int = str.indexOf("Error #");
    var num:String;
    if (numIndex >= 0) {
      num = str.substr(numIndex, 11);
    } else {
      num = str;
    }
    return num;
  }

  /*
  * Functions that pull the error string.
  */
  public static function argumentError( str ){
    return str.slice(0,ARGUMENTERROR.length+4);
  }

  public static function typeError( str ){
    return str.slice(0,TYPEERROR.length+4);
  }

  public static function referenceError( str ){
    return str.slice(0,REFERENCEERROR.length+4);
  }
  public static function rangeError( str ){
    return str.slice(0,RANGEERROR.length+4);
  }
  public static function uriError( str ){
    return str.slice(0,URIERROR.length+4);
  }

  public static function evalError( str ){
    return str.slice(0,EVALERROR.length+4);
  }

  public static function verifyError( str ){
    return str.slice(0,VERIFYERROR.length+4);
  }

  public static function parseError(errorStr:String, len:int):String
  {
    if (errorStr.length > len) {
      errorStr=errorStr.substring(0,len);
    }
    return errorStr;
  }
}


}