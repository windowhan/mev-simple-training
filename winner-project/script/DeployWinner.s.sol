// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import {Winner} from "../src/Winner.sol";

contract DeployWinner is Script {
    function run() external {
        vm.startBroadcast();
        new Winner();
        vm.stopBroadcast();
    }
}
