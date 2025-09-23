/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package ns {

    public class foo {

        Baseball var teamName = "Giants";
        Basketball var teamName = "Kings";
        Hockey var teamName = "Sharks";

        public function getTeam1(){
          return Baseball::teamName;
        }

        public function getTeam2(){
         return Basketball::teamName;
        }

        public function getTeam3(){
         return Hockey::teamName;
        }
    }

}
