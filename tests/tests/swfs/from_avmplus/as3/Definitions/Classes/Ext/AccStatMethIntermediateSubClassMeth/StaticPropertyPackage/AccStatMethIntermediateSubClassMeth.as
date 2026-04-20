/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package StaticPropertyPackage {

    public class AccStatMethIntermediateSubClassMeth extends IntermediateClass {

        public function callEcho(s:String): String {
            return echo(s);
        }

        public function callBaseEcho(s:String): String {
            return BaseClass.echo(s);
        }


    }
}
