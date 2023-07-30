/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
/*
 * Date: 2001-07-13
 *
 * SUMMARY: Applying Function.prototype.call to the Function object itself
 *
 *
 * ECMA-262 15.3.4.4 Function.prototype.call (thisArg [,arg1 [,arg2,…] ] )
 *
 * When applied to the Function object itself, thisArg should be ignored.
 * As explained by Waldemar (waldemar@netscape.com):
 *
 * Function.call(obj, "print(this)") is equivalent to invoking
 * Function("print(this)") with this set to obj. Now, Function("print(this)")
 * is equivalent to new Function("print(this)") (see 15.3.1.1), and the latter
 * ignores the this value that you passed it and constructs a function
 * (which we'll call F) which will print the value of the this that will be
 * passed in when F will be invoked.
 *
 * With the last set of () you're invoking F(), which means you're calling it
 * with no this value. When you don't provide a this value, it defaults to the
 * global object.
 *
 */
//-----------------------------------------------------------------------------
//     var SECTION = "call_001_rt";
//     var VERSION = "";

//     var TITLE   = "Applying Function.prototype.call to the Function object itself";
//     var bug = '(none)';


    var testcases = getTestCases();

function getTestCases() {

    var array = new Array();
    var item = 0;
    
    var UBound = 0;
    var status = '';
    var statusitems = [];
    var actual = '';
    var actualvalues = [];
    var expect= '';
    var expectedvalues = [];
    var self = this; // capture a reference to the global object
    var cnOBJECT_GLOBAL = self.toString();
    var cnOBJECT_OBJECT = (new Object).toString();
    var cnHello = 'Hello';
    var cnRed = 'red';
    var objTEST = {color:cnRed};
    var f = new Function();
    var g = new Function();

    try{
        var f = Function.call(self, 'return cnHello');
       }catch(e:Error){
           thisError=e.toString();
       }finally{
           //print(thisError);
           status = 'Function.call';
           actual = Utils.typeError(thisError);
           expect = 'TypeError: Error #1006';
           //captureThis();
           array[item++] = Assert.expectEq( status, expect, actual);
    } 
 

    /*f = Function.call(self, 'return cnHello');
    
    g = Function.call(objTEST, 'return cnHello');
    
    status = 'Section A of test';
    actual = f();
    expect = cnHello;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);
    
    status = 'Section B of test';
    actual = g();
    expect = cnHello;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);    
    
    f = Function.call(self, 'return this.toString()');
    g = Function.call(objTEST, 'return this.toString()');
    
    status = 'Section C of test';
    actual = f();
    expect = cnOBJECT_GLOBAL;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);    
    
    status = 'Section D of test';
    actual = g();
    expect = cnOBJECT_GLOBAL;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);    
    
    
    f = Function.call(self, 'return this.color');
    g = Function.call(objTEST, 'return this.color');
    
    status = 'Section E of test';
    actual = f();
    expect = undefined;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);    
    
    status = 'Section F of test';
    actual = g();
    expect = undefined;
    //captureThis();
    array[item++] = Assert.expectEq( status, expect, actual);    
    */
    
    return array;
}
/*
function captureThis()
{
  statusitems[UBound] = status;
  actualvalues[UBound] = actual;
  expectedvalues[UBound] = expect;
  UBound++;
}
 */
/*
function test()
{
  enterFunc ('test');
  printBugNumber (bug);
  printStatus (summary);

  for (var i = 0; i < UBound; i++)
  {
    reportCompare(expectedvalues[i], actualvalues[i], statusitems[i]);
    
  }

  exitFunc ('test');
}
 */
