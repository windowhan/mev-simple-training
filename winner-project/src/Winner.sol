pragma solidity ^0.8.19;

contract Winner {
    address public winner;
    function setWinner() external {
        if(winner!=address(0))
            revert("already set winner");
        winner = msg.sender;
    }
}
