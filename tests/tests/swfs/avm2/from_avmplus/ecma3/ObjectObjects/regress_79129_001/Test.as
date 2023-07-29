/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 06 May 2001
 *
 * SUMMARY: Regression test: we shouldn't crash on this code
 *
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=79129
 */
//-------------------------------------------------------------------------------------------------
// var SECTION = "regress_79129_001";
// var VERSION = "";
// var TITLE   = "Regression test: we shouldn't crash on this code";
// var bug = "79129;";

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

    tryThis();

    array[item++] = Assert.expectEq( "Make sure there is no crash", true, true);

    function tryThis()
    {
      obj={};
      obj.a = obj.b = obj.c = 1;
      delete obj.a;
      delete obj.b;
      delete obj.c;
      obj.d = obj.e = 1;
      obj.a=1;
      obj.b=1;
      obj.c=1;
      obj.d=1;
      obj.e=1;
    }

    return array;
}
