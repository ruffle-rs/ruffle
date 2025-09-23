/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package OneOptArgFunction {

    class TestObjInner{

     function returnStringInner(s:String = "inside class inside package",... rest):String { return s; }
     function returnBooleanInner(b:Boolean = true,... rest):Boolean { return b; }
     function returnNumberInner(n:Number = 10,... rest):Number { return n; }
    }

}

