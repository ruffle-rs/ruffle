/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Prototyping {

import com.adobe.test.Assert;
import com.adobe.test.Utils;

  public function test() {
    var defaultObj = new DefaultClass();


    DefaultClass.prototype.array = new Array('a', 'b', 'c');
    Assert.expectEq("Try overriding default property through a default class' prototype object", "1,2,3", defaultObj.accessDefaultProperty().toString());

    DefaultClass.prototype.intNumber = 500;
    Assert.expectEq("Try overriding internal property through a default class' prototype object", "100", defaultObj.intNumber.toString());

    DefaultClass.prototype.protInt = 0; // Note: this override works because the protected property is not visible!
    Assert.expectEq("Try overriding protected property through a default class' prototype object", "0", defaultObj.protInt.toString());

    DefaultClass.prototype.pubUint = 0;
    Assert.expectEq("Try overriding public property through a default class' prototype object", "1", defaultObj.pubUint.toString());

    DefaultClass.prototype.privVar = false;
    Assert.expectEq("Try overriding private property through a default class' prototype object", "true", defaultObj.accPrivProp().toString());

    DefaultClass.prototype.pubStat = 200;
    Assert.expectEq("Try overriding public static property through a default class' prototype object", "100", DefaultClass.pubStat.toString());

    DefaultClass.prototype.nsProp = "fakeNS";
    Assert.expectEq("Try overriding namespace property through a default class' prototype object", "nsProp", defaultObj.accNS().toString());

    DefaultClass.prototype.defaultMethod = false;
    Assert.expectEq("Try overriding default methodsthrough a default class' prototype object", true, defaultObj.defaultMethod());

    DefaultClass.prototype.internalMethod = -1;
    Assert.expectEq("Try overriding internal method through a default class' prototype object", 1, defaultObj.internalMethod());

//DefaultClass.prototype.protectedMethod = -1;
//Assert.expectEq( "Try overriding protected method through a default class' prototype object", 1, defaultObj.protectedMethod() );

    DefaultClass.prototype.publicMethod = false;
    Assert.expectEq("Try overriding public method through a default class' prototype object", true, defaultObj.publicMethod());

    DefaultClass.prototype.privateMethod = false;
    Assert.expectEq("Try overriding private method through a default class' prototype object", true, defaultObj.accPrivMethod());

    DefaultClass.prototype.nsMethod = -1;
    Assert.expectEq("Try overriding namespace method through a default class' prototype object", 1, defaultObj.accNSMethod());

    DefaultClass.prototype.publicFinalMethod = -1;
    Assert.expectEq("Try overriding public final method through a default class' prototype object", 1, defaultObj.publicFinalMethod());

    DefaultClass.prototype.publicStaticMethod = -1;
    Assert.expectEq("Try overriding public static method through a default class' prototype object", 42, DefaultClass.publicStaticMethod());


    DefaultClass.prototype.newArray = new Array('a', 'b', 'c');
    Assert.expectEq("Try adding new property through a default class' prototype object", "a,b,c", defaultObj.newArray.toString());

    DefaultClass.prototype.testFunction = function() {
      return true
    };
    Assert.expectEq("Try adding new method through a default class' prototype object", true, defaultObj.testFunction());

    var equivalent: Boolean = (DefaultClass.prototype.constructor == DefaultClass);
    Assert.expectEq("Verify prototype constructor is equivalent to class object", true, equivalent);


    var thisError10 = "no error thrown";
    var temp: Object = new Object();
    try {
      DefaultClass.prototype = temp;
    } catch (e) {
      thisError10 = e.toString();
    } finally {
      Assert.expectEq("Try to write to FinalClass' prototype object", "ReferenceError: Error #1074",
        Utils.referenceError(thisError10));
    }

    // displays results.

  }
}