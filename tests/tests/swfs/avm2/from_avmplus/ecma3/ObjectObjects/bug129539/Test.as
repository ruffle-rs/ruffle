/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "bug129539";
// var VERSION = "";
// var TITLE   = "";
// var bug = "129539";

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

    var a;

    if( !a ){
        array[item++] = Assert.expectEq( "var a;!a", true, true);
    } else {
        array[item++] = Assert.expectEq( "var a;!a", false, true);
    }

    if( a == null ){
        array[item++] = Assert.expectEq( "var a;a == null", true, true);
    } else {
        array[item++] = Assert.expectEq( "var a;a == null", false, true);
    }

    if( a == undefined ){
        array[item++] = Assert.expectEq( "var a;a == undefined", true, true);
    } else {
        array[item++] = Assert.expectEq( "var a;a == undefined", false, true);
    }

    return array;
}
