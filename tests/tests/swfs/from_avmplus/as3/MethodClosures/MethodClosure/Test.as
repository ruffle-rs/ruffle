/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "MethodClosure";                          // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";                   // Version of JavaScript or ECMA
// var TITLE   = "Method Closure for implementing event handlers";     // Provide ECMA section title or a description
var BUGNUMBER = "";




/*===========================================================================*/

class MethodClosure {
    
    var myMCVar:Number = 50;
    
    function MethodClosureMember () {
        
        return (this.myMCVar)
    }
}

var myMCObj = new MethodClosure();

var myVar = myMCObj.MethodClosureMember;        // This will extract the function and remember 'this'.

var myResult = myVar();                 // This line will now print 50.
Assert.expectEq( "Return a value using method closure: ",  50, myResult );

    

/*===========================================================================*/

              // displays results.
