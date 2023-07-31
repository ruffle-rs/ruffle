/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package Prototyping {



import InternalClassPrototype.*;
import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Class Prototype";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Prototype testing for internal classes";  // Provide ECMA section title or a description
//var BUGNUMBER = "";

public function test() {

    var internalObj = new InternalClass();


    InternalClass.prototype.array = new Array('a', 'b', 'c');
    Assert.expectEq("Try overriding default property through an Internal Class' prototype object", "1,2,3", internalObj.accessDefaultProperty().toString());

    InternalClass.prototype.intNumber = 500;
    Assert.expectEq("Try overriding internal property through an Internal Class' prototype object", "100", internalObj.intNumber.toString());

    InternalClass.prototype.protInt = 0;    // Note: this override works because the protected property is not visible!
    Assert.expectEq("Try overriding protected property through an Internal Class' prototype object", "0", internalObj.protInt.toString());

    InternalClass.prototype.pubUint = 0;
    Assert.expectEq("Try overriding public property through an Internal Class' prototype object", "1", internalObj.pubUint.toString());

    InternalClass.prototype.privVar = false;
    Assert.expectEq("Try overriding private property through an Internal Class' prototype object", "true", internalObj.accPrivProp().toString());

    InternalClass.prototype.pubStat = 200;
    Assert.expectEq("Try overriding public static property through an Internal Class' prototype object", "100", InternalClass.pubStat.toString());

    InternalClass.prototype.nsProp = "fakeNS";
    Assert.expectEq("Try overriding namespace property through an Internal Class' prototype object", "nsProp", internalObj.accNS().toString());

    InternalClass.prototype.defaultMethod = false;
    Assert.expectEq("Try overriding default methodsthrough an Internal Class' prototype object", true, internalObj.defaultMethod());

    InternalClass.prototype.internalMethod = -1;
    Assert.expectEq("Try overriding internal method through an Internal Class' prototype object", 1, internalObj.internalMethod());

//InternalClass.prototype.protectedMethod = -1;
//Assert.expectEq( "Try overriding protected method through an Internal Class' prototype object", 1, internalObj.protectedMethod() );

    InternalClass.prototype.publicMethod = false;
    Assert.expectEq("Try overriding public method through an Internal Class' prototype object", true, internalObj.publicMethod());

    InternalClass.prototype.privateMethod = false;
    Assert.expectEq("Try overriding private method through an Internal Class' prototype object", true, internalObj.accPrivMethod());

    InternalClass.prototype.nsMethod = -1;
    Assert.expectEq("Try overriding namespace method through an Internal Class' prototype object", 1, internalObj.accNSMethod());

    InternalClass.prototype.publicFinalMethod = -1;
    Assert.expectEq("Try overriding public final method through an Internal Class' prototype object", 1, internalObj.publicFinalMethod());

    InternalClass.prototype.publicStaticMethod = -1;
    Assert.expectEq("Try overriding public static method through an Internal Class' prototype object", 42, InternalClass.publicStaticMethod());


    InternalClass.prototype.newArray = new Array('a', 'b', 'c');
    Assert.expectEq("Try adding new property through an internal class' prototype object", "a,b,c", internalObj.newArray.toString());

    InternalClass.prototype.testFunction = function() {
        return true
    };
    Assert.expectEq("Try adding new method through an internal class' prototype object", true, internalObj.testFunction());

    var equivalent: Boolean = (InternalClass.prototype.constructor == InternalClass);
    Assert.expectEq("Verify prototype constructor is equivalent to class object", true, equivalent);


    var thisError10 = "no error thrown";
    var temp: Object = new Object();
    try {
        InternalClass.prototype = temp;
    } catch (e) {
        thisError10 = e.toString();
    } finally {
        Assert.expectEq("Try to write to InternalClass' prototype object", "ReferenceError: Error #1074",
          Utils.referenceError(thisError10));
    }

    // displays results.

}
}
