/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package NamespaceFunctionBody {

    public class TestObj {
        testns function noReturnNoParams() { return "noReturnNoParams"; }
        testns function noReturnParams(s:String, b:Boolean) { return s; }
        testns function noReturnCustomParam(c:Custom) { return new Custom(); }
        testns function returnNoParams():String { return "returnNoParams"; }
        testns function returnParams(s:String, b:Boolean):String { return s; }
        testns function returnCustomNoParams():Custom { return new Custom(); }
    }

}

