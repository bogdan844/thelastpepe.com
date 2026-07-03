// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title The Last Pepe Presale Contract
 * @dev Presale contract for The Last Pepe token on Ethereum
 * @notice Allows users to buy PEPE tokens using ETH, USDT, or USDC
 */
contract PepePresale is ReentrancyGuard, Ownable, Pausable {
    
    // ============ TOKEN CONFIGURATION ============
    IERC20 public pepeToken;
    IERC20 public usdtToken;
    IERC20 public usdcToken;
    
    // ============ PRESALE CONFIGURATION ============
    uint256 public presalePrice = 1000000000000000; // 0.001 USD in wei (10^-3)
    uint256 public totalTokensSold = 0;
    uint256 public presaleCap = 5000000 * 10**18; // 5M PEPE max presale supply
    uint256 public bonusPercentage = 30; // 30% bonus tokens
    
    // ============ TRACKING ============
    mapping(address => uint256) public tokensPurchased;
    mapping(address => uint256) public amountSpent;
    mapping(address => bool) public whitelisteds;
    
    // ============ SETTINGS ============
    address public treasuryWallet;
    bool public requireWhitelist = false;
    
    // ============ EVENTS ============
    event TokensPurchased(
        address indexed buyer,
        address indexed paymentToken,
        uint256 paymentAmount,
        uint256 tokensReceived,
        uint256 bonusTokens
    );
    event PresaleClosed(uint256 totalTokensSold);
    event BonusPercentageUpdated(uint256 newPercentage);
    event WhitelistUpdated(address indexed user, bool status);
    
    // ============ MODIFIERS ============
    modifier onlyWhitelisted() {
        if (requireWhitelist) {
            require(whitelisteds[msg.sender], "Not whitelisted");
        }
        _;
    }
    
    modifier presaleActive() {
        require(totalTokensSold < presaleCap, "Presale cap reached");
        _;
    }
    
    // ============ CONSTRUCTOR ============
    constructor(
        address _pepeToken,
        address _usdtToken,
        address _usdcToken,
        address _treasuryWallet
    ) {
        require(_pepeToken != address(0), "Invalid PEPE token address");
        require(_treasuryWallet != address(0), "Invalid treasury address");
        
        pepeToken = IERC20(_pepeToken);
        usdtToken = IERC20(_usdtToken);
        usdcToken = IERC20(_usdcToken);
        treasuryWallet = _treasuryWallet;
    }
    
    // ============ BUY WITH ETH ============
    /**
     * @dev Buy PEPE tokens with ETH
     */
    function buyWithETH() 
        public 
        payable 
        nonReentrant 
        whenNotPaused 
        onlyWhitelisted 
        presaleActive 
    {
        require(msg.value > 0, "Must send ETH");
        
        // Calculate token amount (18 decimals)
        uint256 pepeAmount = (msg.value * 10**18) / presalePrice;
        
        // Check presale cap
        require(totalTokensSold + pepeAmount <= presaleCap, "Exceeds presale cap");
        
        // Calculate bonus tokens (30% bonus)
        uint256 bonusTokens = (pepeAmount * bonusPercentage) / 100;
        uint256 totalTokens = pepeAmount + bonusTokens;
        
        // Update tracking
        totalTokensSold += pepeAmount; // Only sold amount counts toward cap
        tokensPurchased[msg.sender] += totalTokens;
        amountSpent[msg.sender] += msg.value;
        
        // Transfer PEPE to buyer
        require(
            pepeToken.transfer(msg.sender, totalTokens),
            "Token transfer failed"
        );
        
        // Transfer ETH to treasury
        (bool success, ) = payable(treasuryWallet).call{value: msg.value}("");
        require(success, "ETH transfer failed");
        
        emit TokensPurchased(
            msg.sender,
            address(0), // ETH
            msg.value,
            pepeAmount,
            bonusTokens
        );
    }
    
    // ============ BUY WITH USDT ============
    /**
     * @dev Buy PEPE tokens with USDT (6 decimals)
     * @param amount Amount of USDT (with 6 decimals)
     */
    function buyWithUSDT(uint256 amount) 
        public 
        nonReentrant 
        whenNotPaused 
        onlyWhitelisted 
        presaleActive 
    {
        require(amount > 0, "Amount must be > 0");
        require(address(usdtToken) != address(0), "USDT not configured");
        
        // Convert USDT (6 decimals) to wei equivalent
        uint256 pepeAmount = (amount * 10**12 * 10**18) / presalePrice; // amount * 10^12 to normalize to 18 decimals
        
        // Check presale cap
        require(totalTokensSold + pepeAmount <= presaleCap, "Exceeds presale cap");
        
        // Calculate bonus tokens
        uint256 bonusTokens = (pepeAmount * bonusPercentage) / 100;
        uint256 totalTokens = pepeAmount + bonusTokens;
        
        // Transfer USDT from buyer to contract
        require(
            usdtToken.transferFrom(msg.sender, treasuryWallet, amount),
            "USDT transfer failed"
        );
        
        // Update tracking
        totalTokensSold += pepeAmount;
        tokensPurchased[msg.sender] += totalTokens;
        amountSpent[msg.sender] += amount;
        
        // Transfer PEPE to buyer
        require(
            pepeToken.transfer(msg.sender, totalTokens),
            "Token transfer failed"
        );
        
        emit TokensPurchased(
            msg.sender,
            address(usdtToken),
            amount,
            pepeAmount,
            bonusTokens
        );
    }
    
    // ============ BUY WITH USDC ============
    /**
     * @dev Buy PEPE tokens with USDC (6 decimals)
     * @param amount Amount of USDC (with 6 decimals)
     */
    function buyWithUSDC(uint256 amount) 
        public 
        nonReentrant 
        whenNotPaused 
        onlyWhitelisted 
        presaleActive 
    {
        require(amount > 0, "Amount must be > 0");
        require(address(usdcToken) != address(0), "USDC not configured");
        
        // Convert USDC (6 decimals) to wei equivalent
        uint256 pepeAmount = (amount * 10**12 * 10**18) / presalePrice;
        
        // Check presale cap
        require(totalTokensSold + pepeAmount <= presaleCap, "Exceeds presale cap");
        
        // Calculate bonus tokens
        uint256 bonusTokens = (pepeAmount * bonusPercentage) / 100;
        uint256 totalTokens = pepeAmount + bonusTokens;
        
        // Transfer USDC from buyer to contract
        require(
            usdcToken.transferFrom(msg.sender, treasuryWallet, amount),
            "USDC transfer failed"
        );
        
        // Update tracking
        totalTokensSold += pepeAmount;
        tokensPurchased[msg.sender] += totalTokens;
        amountSpent[msg.sender] += amount;
        
        // Transfer PEPE to buyer
        require(
            pepeToken.transfer(msg.sender, totalTokens),
            "Token transfer failed"
        );
        
        emit TokensPurchased(
            msg.sender,
            address(usdcToken),
            amount,
            pepeAmount,
            bonusTokens
        );
    }
    
    // ============ VIEW FUNCTIONS ============
    
    /**
     * @dev Get the amount of PEPE tokens for a given payment amount
     * @param paymentAmount Amount being paid
     * @return pepeAmount Amount of PEPE tokens (including bonus)
     */
    function getPepeAmount(uint256 paymentAmount) 
        public 
        view 
        returns (uint256) 
    {
        uint256 baseAmount = (paymentAmount * 10**18) / presalePrice;
        uint256 bonus = (baseAmount * bonusPercentage) / 100;
        return baseAmount + bonus;
    }
    
    /**
     * @dev Get remaining tokens available for presale
     */
    function getRemainingTokens() public view returns (uint256) {
        return presaleCap > totalTokensSold ? presaleCap - totalTokensSold : 0;
    }
    
    /**
     * @dev Get presale progress percentage
     */
    function getPresaleProgress() public view returns (uint256) {
        return (totalTokensSold * 100) / presaleCap;
    }
    
    /**
     * @dev Get user's purchase info
     */
    function getUserPurchaseInfo(address user) 
        public 
        view 
        returns (uint256 pepeTokens, uint256 usdSpent) 
    {
        return (tokensPurchased[user], amountSpent[user]);
    }
    
    // ============ OWNER FUNCTIONS ============
    
    /**
     * @dev Update bonus percentage
     */
    function setBonusPercentage(uint256 _newPercentage) public onlyOwner {
        require(_newPercentage <= 100, "Invalid percentage");
        bonusPercentage = _newPercentage;
        emit BonusPercentageUpdated(_newPercentage);
    }
    
    /**
     * @dev Update presale price
     */
    function setPresalePrice(uint256 _newPrice) public onlyOwner {
        require(_newPrice > 0, "Price must be > 0");
        presalePrice = _newPrice;
    }
    
    /**
     * @dev Update presale cap
     */
    function setPresaleCap(uint256 _newCap) public onlyOwner {
        presaleCap = _newCap;
    }
    
    /**
     * @dev Add or remove user from whitelist
     */
    function setWhitelist(address user, bool status) public onlyOwner {
        whitelisteds[user] = status;
        emit WhitelistUpdated(user, status);
    }
    
    /**
     * @dev Add multiple users to whitelist
     */
    function batchWhitelist(address[] calldata users, bool status) 
        public 
        onlyOwner 
    {
        for (uint256 i = 0; i < users.length; i++) {
            whitelisteds[users[i]] = status;
            emit WhitelistUpdated(users[i], status);
        }
    }
    
    /**
     * @dev Enable/disable whitelist requirement
     */
    function setRequireWhitelist(bool _require) public onlyOwner {
        requireWhitelist = _require;
    }
    
    /**
     * @dev Pause presale
     */
    function pausePresale() public onlyOwner {
        _pause();
    }
    
    /**
     * @dev Unpause presale
     */
    function unpausePresale() public onlyOwner {
        _unpause();
    }
    
    /**
     * @dev Update treasury wallet
     */
    function setTreasuryWallet(address _newTreasury) public onlyOwner {
        require(_newTreasury != address(0), "Invalid address");
        treasuryWallet = _newTreasury;
    }
    
    /**
     * @dev Withdraw remaining PEPE tokens (after presale ends)
     */
    function withdrawRemainingTokens() public onlyOwner {
        uint256 balance = pepeToken.balanceOf(address(this));
        require(balance > 0, "No tokens to withdraw");
        require(pepeToken.transfer(owner(), balance), "Withdrawal failed");
    }
    
    /**
     * @dev Emergency withdrawal of tokens
     */
    function emergencyWithdraw(address token) public onlyOwner {
        uint256 balance = IERC20(token).balanceOf(address(this));
        require(balance > 0, "No balance");
        require(IERC20(token).transfer(owner(), balance), "Transfer failed");
    }
}
