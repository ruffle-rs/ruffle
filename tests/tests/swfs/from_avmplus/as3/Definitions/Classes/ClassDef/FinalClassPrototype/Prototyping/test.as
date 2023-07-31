package Prototyping {


import FinalClassPrototype.*;

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Class Prototype";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Prototype testing for final classes";  // Provide ECMA section title or a description
//var BUGNUMBER = "";

public function test() {

    var finalObj = new FinalClass();


    FinalClass.prototype.array = new Array('a', 'b', 'c');
    Assert.expectEq("Try overriding default property through a Final Class' prototype object", "1,2,3", finalObj.accessDefaultProperty().toString());

    FinalClass.prototype.intNumber = 500;
    Assert.expectEq("Try overriding internal property through a Final Class' prototype object", "100", finalObj.intNumber.toString());

    FinalClass.prototype.protInt = 0;   // Note: this works because the protected property is not visible!
    Assert.expectEq("Try overriding protected property through a Final Class' prototype object", "0", finalObj.protInt.toString());

    FinalClass.prototype.pubUint = 0;
    Assert.expectEq("Try overriding public property through a Final Class' prototype object", "1", finalObj.pubUint.toString());

    FinalClass.prototype.privVar = false;
    Assert.expectEq("Try overriding private property through a Final Class' prototype object", "true", finalObj.accPrivProp().toString());

    FinalClass.prototype.pubStat = 200;
    Assert.expectEq("Try overriding public static property through a Final Class' prototype object", "100", FinalClass.pubStat.toString());

    FinalClass.prototype.nsProp = "fakeNS";
    Assert.expectEq("Try overriding namespace property through a Final Class' prototype object", "nsProp", finalObj.accNS().toString());

    FinalClass.prototype.defaultMethod = false;
    Assert.expectEq("Try overriding default methodsthrough a Final Class' prototype object", true, finalObj.defaultMethod());

    FinalClass.prototype.internalMethod = -1;
    Assert.expectEq("Try overriding internal method through a Final Class' prototype object", 1, finalObj.internalMethod());

//FinalClass.prototype.protectedMethod = -1;
//Assert.expectEq( "Try overriding protected method through a Final Class' prototype object", 1, finalObj.protectedMethod() );

    FinalClass.prototype.publicMethod = false;
    Assert.expectEq("Try overriding public method through a Final Class' prototype object", true, finalObj.publicMethod());

    FinalClass.prototype.privateMethod = false;
    Assert.expectEq("Try overriding private method through a Final Class' prototype object", true, finalObj.accPrivMethod());

    FinalClass.prototype.nsMethod = -1;
    Assert.expectEq("Try overriding namespace method through a Final Class' prototype object", 1, finalObj.accNSMethod());

    FinalClass.prototype.publicFinalMethod = -1;
    Assert.expectEq("Try overriding public final method through a Final Class' prototype object", 1, finalObj.publicFinalMethod());

    FinalClass.prototype.publicStaticMethod = -1;
    Assert.expectEq("Try overriding public static method through a Final Class' prototype object", 42, FinalClass.publicStaticMethod());


    FinalClass.prototype.newArray = new Array('a', 'b', 'c');
    Assert.expectEq("Try adding new property through a final class' prototype object", "a,b,c", finalObj.newArray.toString());

    FinalClass.prototype.testFunction = function() {
        return true
    };
    Assert.expectEq("Try adding new method through a final class' prototype object", true, finalObj.testFunction());

    var equivalent: Boolean = (FinalClass.prototype.constructor == FinalClass);
    Assert.expectEq("Verify prototype constructor is equivalent to class object", true, equivalent);


    var thisError10 = "no error thrown";
    var temp: Object = new Object();
    try {
        FinalClass.prototype = temp;
    } catch (e) {
        thisError10 = e.toString();
    } finally {
        Assert.expectEq("Try to write to FinalClass' prototype object", "ReferenceError: Error #1074",
          Utils.referenceError(thisError10));
    }

    // displays results.

}
}