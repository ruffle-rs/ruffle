/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Regression Tests";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Bug 415080";       // Provide ECMA section title or a description



class Bug415080 {

    public function Bug415080() {

    }

    public function _badFunction(a : String = 'b') : Boolean {
        switch (true) {
            case true :
                a.indexOf('e');
                if(true == true) {
                    return true;
                }
        }
        return false;
    }

}

var t = new Bug415080();


Assert.expectEq('No verifier error when running _badFunction', true, t._badFunction());

