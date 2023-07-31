/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Prototyping {


import PublicClassPrototype.*;

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Class Prototype";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing prototype for public classes";  // Provide ECMA section title or a description
//var BUGNUMBER = "";

public function test() {

    var publicObj = new PublicClass();


    PublicClass.prototype.array = new Array('a', 'b', 'c');
    Assert.expectEq("Try overriding default property through a public class' prototype object", "1,2,3", publicObj.accessDefaultProperty().toString());

    PublicClass.prototype.intNumber = 500;
    Assert.expectEq("Try overriding internal property through a public class' prototype object", "100", publicObj.intNumber.toString());

    PublicClass.prototype.protInt = 0;  // Note: this override works because the protected property is not visible!
    Assert.expectEq("Try overriding protected property through a public class' prototype object", "0", publicObj.protInt.toString());

    PublicClass.prototype.pubUint = 0;
    Assert.expectEq("Try overriding public property through a public class' prototype object", "1", publicObj.pubUint.toString());

    PublicClass.prototype.privVar = false;
    Assert.expectEq("Try overriding private property through a public class' prototype object", "true", publicObj.accPrivProp().toString());

    PublicClass.prototype.pubStat = 200;
    Assert.expectEq("Try overriding public static property through a public class' prototype object", "100", PublicClass.pubStat.toString());

    PublicClass.prototype.nsProp = "fakeNS";
    Assert.expectEq("Try overriding namespace property through a public class' prototype object", "nsProp", publicObj.accNS().toString());

    PublicClass.prototype.defaultMethod = false;
    Assert.expectEq("Try overriding default methodsthrough a public class' prototype object", true, publicObj.defaultMethod());

    PublicClass.prototype.internalMethod = -1;
    Assert.expectEq("Try overriding internal method through a public class' prototype object", 1, publicObj.internalMethod());

//PublicClass.prototype.protectedMethod = -1;
//Assert.expectEq( "Try overriding protected method through a public class' prototype object", 1, publicObj.protectedMethod() );

    PublicClass.prototype.publicMethod = false;
    Assert.expectEq("Try overriding public method through a public class' prototype object", true, publicObj.publicMethod());

    PublicClass.prototype.privateMethod = false;
    Assert.expectEq("Try overriding private method through a public class' prototype object", true, publicObj.accPrivMethod());

    PublicClass.prototype.nsMethod = -1;
    Assert.expectEq("Try overriding namespace method through a public class' prototype object", 1, publicObj.accNSMethod());

    PublicClass.prototype.publicFinalMethod = -1;
    Assert.expectEq("Try overriding public final method through a public class' prototype object", 1, publicObj.publicFinalMethod());

    PublicClass.prototype.publicStaticMethod = -1;
    Assert.expectEq("Try overriding public static method through a public class' prototype object", 42, PublicClass.publicStaticMethod());


    PublicClass.prototype.newArray = new Array('a', 'b', 'c');
    Assert.expectEq("Try adding new property through a public class' prototype object", "a,b,c", publicObj.newArray.toString());

    PublicClass.prototype.testFunction = function() {
        return true
    };
    Assert.expectEq("Try adding new method through a public class' prototype object", true, publicObj.testFunction());

    var equivalent: Boolean = (PublicClass.prototype.constructor == PublicClass);
    Assert.expectEq("Verify prototype constructor is equivalent to class object", true, equivalent);


    var thisError10 = "no error thrown";
    var temp: Object = new Object();
    try {
        PublicClass.prototype = temp;
    } catch (e: ReferenceError) {
        thisError10 = e.toString();
    } finally {
        Assert.expectEq("Try to write to PublicClass' prototype object", "ReferenceError: Error #1074",
          Utils.referenceError(thisError10));
    }

    // displays results.

}
}
