/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "instanceof-001";
//     var VERSION = "ECMA_2";
//     var TITLE   = "instanceof"


    var testcases = getTestCases();
    
    function InstanceOf( object_1, object_2, expect, array, item ) {
        result = object_1 instanceof object_2;

        array[item] = Assert.expectEq(
     
            "(" + object_1 + ") instanceof " + object_2,
            expect,
            result );
    }

function getTestCases() {
    var array = new Array();
    var item = 0;
    
    function Gen3(value) {
        this.value = value;
        this.generation = 3;
        this.toString = function (){return "Gen" + this.generation+ " instance"};
    }
    Gen3.name = 3;
    var origPrototypeToString = Gen3.constructor.prototype.toString;
    Gen3.constructor.prototype.toString = function(){return this.name+" object"};

    function Gen2(value) {
        this.value = value;
        this.generation = 2;
    }
    Gen2.name = 2;
    Gen2.prototype = new Gen3();

    function Gen1(value) {
        this.value = value;
        this.generation = 1;
    }
    Gen1.name = 1;
    Gen1.prototype = new Gen2();

    function Gen0(value) {
        this.value = value;
        this.generation = 0;
    }
    Gen0.name = 0;
    Gen0.prototype = new Gen1();


    function GenA(value) {
        this.value = value;
        this.generation = "A";
        this.toString = function (){return "instance of Gen" +this.generation};

    }
    GenA.prototype = new Gen0();
    GenA.name = "A";

    function GenB(value) {
        this.value = value;
        this.generation = "B";
        this.toString = function (){ return "instance of Gen"+this.generation};
    }
    GenB.name = "B"
    //GenB.prototype = void 0;

    // RelationalExpression is not an object.

    InstanceOf( true, Boolean, true, array, item++ );
    InstanceOf( new Boolean(false), Boolean, true, array, item++ );

    // Identifier is a function, prototype of Identifier is not an object

    // RelationalExpression.__proto__ ==  (but not ===) Identifier.prototype
    InstanceOf( new Gen2(), Gen0, false, array, item++ );
    InstanceOf( new Gen2(), Gen1, false, array, item++ );
    InstanceOf( new Gen2(), Gen2, true, array, item++ );
    InstanceOf( new Gen2(), Gen3, true, array, item++ );

    // RelationalExpression.__proto__.__proto__ === Identifier.prototype
    InstanceOf( new Gen0(), Gen0, true, array, item++ );
    InstanceOf( new Gen0(), Gen1, true, array, item++ );
    InstanceOf( new Gen0(), Gen2, true, array, item++ );
    InstanceOf( new Gen0(), Gen3, true, array, item++ );

    InstanceOf( new Gen0(), Object, true, array, item++ );
    InstanceOf( new Gen0(), Function, false, array, item++ );

    InstanceOf( Gen0, Function, true, array, item++ );
    InstanceOf( Gen0, Object, true, array, item++ );

    //restore
    Gen3.constructor.prototype.toString = origPrototypeToString;
    return array;
}
