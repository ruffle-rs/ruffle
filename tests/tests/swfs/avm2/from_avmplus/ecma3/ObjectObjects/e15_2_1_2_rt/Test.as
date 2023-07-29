/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.2.1.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Object()";


    var testcases = getTestCases();

   // TODO: REVIEW AS4 CONVERSION ISSUE 
 // Adding this function getJSClass directly to file rather than in Utils

 function getJSClass(obj)
{
  if (isObject(obj))
    return findClass(findType(obj));
  return cnNoObject;
}
function isObject(obj)
{
  return obj instanceof Object;
}

function findType(obj)
{
  var cnObjectToString = Object.prototype.toString;
  return cnObjectToString.apply(obj);
}
// given '[object Number]',  return 'Number'
function findClass(sType)
{
  var re =  /^\[.*\s+(\w+)\s*\]$/;
  var a = sType.match(re);

  if (a && a[1])
    return a[1];
  return cnNoClass;
}

function getTestCases() {
    var array = new Array();
    var item = 0;

    var MYOB = Object();

           array[item++] = Assert.expectEq(  "var MYOB = Object(); MYOB.valueOf()", MYOB,MYOB.valueOf());



    array[item++] = Assert.expectEq(  "typeof Object()",       "object",               typeof (Object(null)) );


           array[item++] = Assert.expectEq(  "var MYOB = Object();  MYOB.toString()", "[object Object]",MYOB.toString());



    return ( array );
}
