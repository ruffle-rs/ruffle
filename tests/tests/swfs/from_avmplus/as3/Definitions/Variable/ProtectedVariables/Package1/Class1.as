/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package Package1
{
    public class Class1
    {
        protected var classItem1 = "Class1 protected var classItem1 set at creation time";
        protected const classItem2 = "Class1 protected const classItem2 set at creation time";
        protected static var classItem3 = "Class1 protected static var classItem3 set at creation time";
        protected static const classItem4 = "Class protected static const classItem4 set at creation time";

        public function setClassItem1(msg)
        {
            classItem1 = msg;
        }

        public function getClassItem1()
        {
            return classItem1;
        }

        public function getClassItem2()
        {
            return classItem2;
        }

        public static function setClassItem3(msg)
        {
            classItem3 = msg;
        }

        public static function getClassItem3()
        {
            return classItem3;
        }

        public static function getClassItem4()
        {
            return classItem4;
        }
    }
}
