// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol"; // Assuming you have the corresponding ImageID contract

contract PredictionGame {
    IRiscZeroVerifier private verifier;
    bytes32 public constant predictionImageId = ImageID.PREDICTION_ID;

    enum ContractState {PREDICTION_PERIOD, AWAITING_EVENT, AWAITING_RESULT}
    ContractState public currentState;

    uint256 public predictionPeriodEnds;
    uint256 public eventPeriodEnds;

    mapping(address => uint256) public predictions;
    mapping(uint256 => address[]) private predictionResults;
    uint256 public predictionCost;
    uint256 private prizePool;
    uint256 private predictionResult;

    constructor(IRiscZeroVerifier _verifier, uint256 _predictionDuration, uint256 _eventDuration, uint256 _predictionCost) {
        verifier = _verifier;
        currentState = ContractState.PREDICTION_PERIOD;
        predictionPeriodEnds = block.timestamp + _predictionDuration;
        eventPeriodEnds = predictionPeriodEnds + _eventDuration;
        predictionCost = _predictionCost;
    }

    function makePrediction(uint256 _prediction) external payable {
        require(currentState == ContractState.PREDICTION_PERIOD, "Prediction period over");
        require(msg.value == predictionCost, "Incorrect value sent");
        predictions[msg.sender] = _prediction;
        predictionResults[_prediction].push(msg.sender);
        prizePool += msg.value;
    }

    function updateState() public {
        if (currentState == ContractState.PREDICTION_PERIOD && block.timestamp >= predictionPeriodEnds) {
            currentState = ContractState.AWAITING_EVENT;
        } else if (currentState == ContractState.AWAITING_EVENT && block.timestamp >= eventPeriodEnds) {
            currentState = ContractState.AWAITING_RESULT;
        }
    }

    function triggerResult(uint256 _result, bytes32 postStateDigest, bytes calldata seal) external {
        require(currentState == ContractState.AWAITING_RESULT, "Not awaiting results");
        bytes memory journal = abi.encode(_result);
        require(verifier.verify(seal, predictionImageId, postStateDigest, sha256(journal)), "Verification failed");
        predictionResult = _result;
        distributePrize();
    }

    function distributePrize() private {
        address[] memory winners = predictionResults[predictionResult];
        require(winners.length > 0, "No winners");
        uint256 winnerPrize = prizePool / winners.length;
        for (uint i = 0; i < winners.length; i++) {
            payable(winners[i]).transfer(winnerPrize);
        }
        // Reset after distributing prizes
        resetGame();
    }

    function resetGame() private {
        currentState = ContractState.PREDICTION_PERIOD;
        predictionPeriodEnds = block.timestamp + (eventPeriodEnds - predictionPeriodEnds);
        eventPeriodEnds = predictionPeriodEnds + (eventPeriodEnds - predictionPeriodEnds);
        prizePool = 0;
    }
}