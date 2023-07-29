/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "instanceof-003_TypeErrors";
// var VERSION = "ECMA_2";
// var TITLE   = "instanceof"


var testcases = getTestCases();

function InstanceOf( object_1, object_2, expect, array, item ) {

    try{
        result = object_1 instanceof object_2;
    } catch (e:TypeError) {
        result = e.toString();
    } finally {
        array[item] = Assert.expectEq(
             
            "(" + object_1 + ") instanceof " + object_2,
            expect,
            Utils.typeError(result));
    }

}

function getTestCases() {
    var array = new Array();
    var item = 0;
    // RHS of instanceof must be a Class or Function
    var RHSTypeError="TypeError: Error #1040";
    
    //Boolean
    InstanceOf( true, true, RHSTypeError, array, item++ );
    InstanceOf( false, false, RHSTypeError, array, item++ );
    InstanceOf( false, true, RHSTypeError, array, item++ );
    
    InstanceOf( new Boolean(), true, RHSTypeError, array, item++ );
    InstanceOf( new Boolean(true), true, RHSTypeError, array, item++ );
    InstanceOf( new Boolean(false), true, RHSTypeError, array, item++ );
    
    InstanceOf( new Boolean(), false, RHSTypeError, array, item++ );
    InstanceOf( new Boolean(true), false, RHSTypeError, array, item++ );
    InstanceOf( new Boolean(false), false, RHSTypeError, array, item++ );
    
    InstanceOf( new Boolean(), new Boolean(), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(true), new Boolean(), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(false), new Boolean(), RHSTypeError, array, item++ );
    
    InstanceOf( new Boolean(), new Boolean(true), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(true), new Boolean(true), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(false), new Boolean(true), RHSTypeError, array, item++ );
    
    InstanceOf( new Boolean(), new Boolean(false), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(true), new Boolean(false), RHSTypeError, array, item++ );
    InstanceOf( new Boolean(false), new Boolean(false), RHSTypeError, array, item++ );
    
    //Number
    InstanceOf( 4, 3, RHSTypeError, array, item++ );
    InstanceOf( new Number(), 3, RHSTypeError, array, item++ );
    InstanceOf( new Number(4), 3, RHSTypeError, array, item++ );
    InstanceOf( new Number(), new Number(), RHSTypeError, array, item++ );
    InstanceOf( new Number(4), new Number(4), RHSTypeError, array, item++ );
    
    
    //String
    InstanceOf( "test", "test", RHSTypeError, array, item++ );
    InstanceOf( "test", new String("test"), RHSTypeError, array, item++ );
    InstanceOf( "test", new String(), RHSTypeError, array, item++ );
    InstanceOf( "test", new String(""), RHSTypeError, array, item++ );
    InstanceOf( new String(), new String("test"), RHSTypeError, array, item++ );
    InstanceOf( new String(""), new String(), RHSTypeError, array, item++ );
    InstanceOf( new String("test"), new String(""), RHSTypeError, array, item++ );

    return array;
}
