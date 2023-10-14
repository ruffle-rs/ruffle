/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/*
 * Date: 14 October 2001
 *
 * SUMMARY: Regression test for Bugzilla bug 104584
 * See http://bugzilla.mozilla.org/show_bug.cgi?id=104584
 *
 * Testing that we don't crash on this code. The idea is to
 * call F,G WITHOUT providing an argument. This caused a crash
 * on the second call to obj.toString() or print(obj) below -
 */
//-----------------------------------------------------------------------------
// var SECTION = "regress_104584";       // provide a document reference (ie, ECMA section)
// var VERSION = "";  // Version of JavaScript or ECMA
// var TITLE   = "Testing that we don't crash on this code -";       // Provide ECMA section title or a description
var BUGNUMBER = "104584";


var testcases = getTestCases()

function getTestCases() {
    var array = new Array();
    var item = 0;

    F();
    G();
    
    function F(obj)
    {
      if(!obj)
        obj = {};
      obj.toString();
      //gc();
      obj.toString();
    }
    
    
    function G(obj)
    {
      if(!obj)
        obj = {};
      trace(obj);
      //gc();
      trace(obj);
    }
    
    array[item++] = Assert.expectEq( "Make sure we don't crash", true, true);
    
    return array;
}
