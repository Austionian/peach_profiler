default:
  just -l

alias t := test

test:
  #!/bin/bash

  # test all the different feature combinations
  cargo hack test --each-feature
