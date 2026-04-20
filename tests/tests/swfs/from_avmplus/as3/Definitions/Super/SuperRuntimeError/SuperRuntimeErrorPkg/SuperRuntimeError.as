/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// test a variety of super error conditions
package SuperRuntimeErrorPkg {
    import SuperRuntimeErrorPackage.*;

    public class SuperRuntimeError extends base {
        private function noSuchFunction() : String {
            return "I should not be called!";
        }
        public function missingSuperMethod() : String {
            return super.noSuchFunction();
        }
        public function callSuperPrivate() : String {
            return super.privFunc();
        }
        public function callSuperOtherPackage() : String {
            return super.otherPackageFunc();
        }
    }
}
