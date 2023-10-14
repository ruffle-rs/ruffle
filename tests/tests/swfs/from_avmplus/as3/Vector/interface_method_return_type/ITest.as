/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {

public interface ITest {
  function get vo(): Vector.<Object>;

  function get vs(): Vector.<String>;

  function get vi(): Vector.<int>;

  function get vc(): Vector.<C>;

  function get va(): Vector.<*>;
}
}