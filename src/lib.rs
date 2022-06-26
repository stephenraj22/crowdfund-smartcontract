// Author: Stephen Raj D
// Near metabuild hackathon

use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    serde::{Deserialize, Serialize},
    AccountId, PanicOnDefault, Promise,
};
use near_sdk::{env, near_bindgen};
use std::collections::LinkedList;


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Donar {
    account_id:AccountId,
    amount:u16,
    created_at:u64,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Crowdfund {
    account_id: AccountId,
    cf_name:String,
    cf_desc:String,
    deadline : u64,        
    created_at:u64,
    target_value:u16,
    min_amount:u8,
    amount_raised:u16,
    active:bool,
    withdraw:bool,
    donars:LinkedList<Donar>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Nft {
    owner: AccountId,
    uri: String,
    category: String,
    desc:String,
    price:u64,
}

#[derive(Serialize,Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ActiveCrowdfunds {
    crowdfunds: Vec<Crowdfund>,
    cf_ids:Vec<u16>,
}
#[derive(Serialize,Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MyCrowdfunds {
    crowdfunds: Vec<Crowdfund>,
    cf_ids:Vec<u16>,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    crowdfunds: LookupMap<u16, Crowdfund>,
    nfts: LookupMap<u16, Nft>,
    active_cfs: UnorderedSet<u16>,
    inactive_cfs: UnorderedSet<u16>,
    active_accounts:UnorderedSet<AccountId>,
    count:u16,
    nft_count:u16,
}

#[near_bindgen]
impl Contract {
    // ADD CONTRACT METHODS HERE
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            crowdfunds: LookupMap::new(b"c"),
            nfts: LookupMap::new(b"n"),
            active_cfs: UnorderedSet::new(b"a"),
            active_accounts: UnorderedSet::new(b"o"),
            inactive_cfs: UnorderedSet::new(b"i"),
            count:0,
            nft_count:0,
        }
    }

    #[payable]
    pub fn mint(
        &mut self,
        uri: String,
        category: String,
        desc:String,
        price:u64,
    )
    {
        let existing = self.nfts.insert(
            &self.nft_count, 
            &Nft{
                owner:env::predecessor_account_id(),
                uri: uri,
                category: category,
                desc:desc,
                price:price,
            }
        );
        assert!(existing.is_none(), "NFT with that key already exists");
        self.nft_count += 1;

    }

    #[payable]
    pub fn make_owner(
        &mut self,
        nft_id:u16,
    ){
        let mut nft = self
        .nfts
        .get(&nft_id)
        .expect("NO_NFT_FOUND");
        if nft.owner != env::predecessor_account_id() {
            let old_owner = nft.owner;
            let base:u128 = 10;
            let amt:u128 = nft.price.into();
            nft.owner = env::predecessor_account_id();
            Promise::new(old_owner).transfer(amt*base.pow(24));
        }
    }

    pub fn all_nfts(&self) -> Vec<Nft>{
        let mut nfts = vec![];
        for nft_id in 0..self.nft_count {
            let nft = self
            .nfts
            .get(&nft_id)
            .unwrap_or_else(|| env::panic_str("ERR_LOADING_NFT"));
            nfts.push(nft);
        }
        return nfts;
    }

    pub fn nfts_by_owner(&self, owner_id:AccountId) -> Vec<Nft>{
        let mut nfts = vec![];
        for nft_id in 0..self.nft_count {
            let nft = self
            .nfts
            .get(&nft_id)
            .unwrap_or_else(|| env::panic_str("ERR_LOADING_NFT"));
            if nft.owner == owner_id{
                nfts.push(nft);
            }
        }
        return nfts;
    }

    pub fn get_nft(&self, nft_id:u16) -> Nft{
        let mut nft = self
        .nfts
        .get(&nft_id)
        .unwrap_or_else(|| env::panic_str("ERR_LOADING_CF"));
        return nft;
    }

    #[payable]
    pub fn create_cf(
        &mut self,
        account_id:AccountId, 
        cf_name:String, 
        cf_desc:String, 
        deadline:u64,
        created_at:u64, 
        target_value:u16,
        min_amount:u8,
    ){
        let active_accounts = self.active_accounts.to_vec();
        for cf_account in active_accounts {
            assert!(cf_account != account_id, "Account has active crowdfund");
        }
        let account_id_copy = account_id.clone();
        let existing = self.crowdfunds.insert(
            &self.count,
            &Crowdfund {
                account_id: account_id,
                cf_name:cf_name,
                cf_desc:cf_desc,
                deadline : deadline,
                created_at: created_at,
                target_value:target_value,
                min_amount:min_amount,
                amount_raised:0,
                active:true,
                withdraw:false,
                donars:LinkedList::new(),
            }
        );
        assert!(existing.is_none(), "Crowdfund with that key already exists");
        self.active_cfs.insert(&self.count);
        self.active_accounts.insert(&account_id_copy);
        self.count += 1;
    }

    #[payable]
    pub fn contribute(
        &mut self,
        account_id:AccountId, 
        amount:u16,
        cf_id:u16,
        created_at:u64,
    ){
        let cf_id_copy = cf_id.clone();
        let mut cf = self
            .crowdfunds
            .get(&cf_id)
            .expect("NO_CF_FOUND");
        let account_id_copy = account_id.clone();
        if cf.active {
            cf.amount_raised += amount;
            cf.donars.push_back(Donar {
                account_id: account_id,
                amount: amount, 
                created_at: created_at
            });
            if cf.amount_raised >=  cf.target_value {
                self.active_cfs.remove(&cf_id);
                self.active_accounts.remove(&cf.account_id);
                self.inactive_cfs.insert(&cf_id_copy);
                cf.active = false;
            }
            self.crowdfunds.insert(&cf_id, &cf);
        }
    }

    #[payable]
    pub fn withdraw(
        &mut self,
        cf_id:u16,
    ){
        let account_id = env::predecessor_account_id();
        let mut cf = self
            .crowdfunds
            .get(&cf_id)
            .expect("NO_CF_FOUND");
        println!("{},{},{},{}", account_id, cf.account_id, cf.active, cf.withdraw);
        if account_id == cf.account_id && !cf.active && !cf.withdraw{
            cf.withdraw = true;
            self.crowdfunds.insert(&cf_id, &cf);
            let base:u128 = 10;
            let amt:u128 = cf.amount_raised.into();
            println!("{}", (amt*base.pow(24)).to_string());
            Promise::new(env::predecessor_account_id()).transfer(amt*base.pow(24));
        }
    }

    pub fn get_cf(&self, cf_id:u16) -> Crowdfund{
        let cf = self.crowdfunds.get(&cf_id).expect("NO_CF_FOUND");
        Crowdfund{
            account_id: cf.account_id,
            cf_name:cf.cf_name,
            cf_desc:cf.cf_desc,
            deadline : cf.deadline,
            created_at: cf.created_at,
            target_value:cf.target_value,
            min_amount:cf.min_amount,
            amount_raised:cf.amount_raised,
            active:cf.active,
            withdraw:cf.withdraw,
            donars:cf.donars,
        }
    }

    pub fn get_active_cfs(&self) -> ActiveCrowdfunds{
        let active_cfs = self.active_cfs.to_vec();
        let mut all_active_cfs = vec![];
        let mut active_cf_ids = vec![];
        for cf_id in active_cfs {
            let mut cf = self
                .crowdfunds
                .get(&cf_id)
                .unwrap_or_else(|| env::panic_str("ERR_LOADING_CF"));
                all_active_cfs.push(cf);
                active_cf_ids.push(cf_id);
        }
        ActiveCrowdfunds {
            crowdfunds:all_active_cfs,
            cf_ids:active_cf_ids
        }
    }
    pub fn get_active_accounts(&self) -> Vec<AccountId>{
        let active_accounts = self.active_accounts.to_vec();
        return active_accounts;
    }

    pub fn get_inactive_cfs(&self) -> ActiveCrowdfunds{
        let inactive_cfs = self.inactive_cfs.to_vec();
        let mut all_inactive_cfs = vec![];
        let mut inactive_cf_ids = vec![];
        for cf_id in inactive_cfs {
            let mut cf = self
                .crowdfunds
                .get(&cf_id)
                .unwrap_or_else(|| env::panic_str("ERR_LOADING_CF"));
                all_inactive_cfs.push(cf);
                inactive_cf_ids.push(cf_id);
        }
        ActiveCrowdfunds {
            crowdfunds:all_inactive_cfs,
            cf_ids:inactive_cf_ids
        }
    }

    pub fn get_cfs_by_accountId(&self, account_id:AccountId) -> MyCrowdfunds {
        let mut my_cfs = vec![];
        let mut cf_ids = vec![];
        for cf_id in 0..self.count {
            let mut cf = self
                .crowdfunds
                .get(&cf_id)
                .unwrap_or_else(|| env::panic_str("ERR_LOADING_CF"));
            if cf.account_id == account_id {
                my_cfs.push(cf);
                cf_ids.push(cf_id)
            }
        }
        MyCrowdfunds {
            crowdfunds:my_cfs,
            cf_ids:cf_ids,
        }
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // TESTS HERE
    #[test]
    fn check_withdraw(){
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        let context = get_context(alice.clone());
        testing_env!(context.build());

        let mut contract = Contract::new(alice);
        let account_id =  AccountId::new_unchecked("alice.testnet".to_string());
        contract.create_cf(
            account_id,
            "flood relief".to_string(),
            "from India".to_string(),
            1645488000000000,
            1645488000000001,
            1,
            1,
        );
        let account_id1 =  AccountId::new_unchecked("alice1.testnet".to_string());
        contract.create_cf(
            account_id1,
            "drought relief".to_string(),
            "from India".to_string(),
            1645488000000000,
            1645488000000001,
            40,
            1,
        );
        let account_id2 =  AccountId::new_unchecked("alice1.testnet".to_string());
        contract.contribute(account_id2,1,0,1);
        let account_id2 =  AccountId::new_unchecked("alice1.testnet".to_string());
        contract.contribute(account_id2,1,1,1);
        let account_id2 =  AccountId::new_unchecked("alice.testnet".to_string());
        contract.withdraw( 0);
        let cf = contract.get_cf(0);
        println!("Let's debug contribute-0: {:?}", cf);
        let cf = contract.get_cf(1);
        println!("Let's debug contribute-1: {:?}", cf);
        let cfs = contract.get_active_cfs();
        println!("Let's debug active cfs-3: {:?}", cfs);
        let account_id2 =  AccountId::new_unchecked("alice.testnet".to_string());
        let cfs = contract.get_cfs_by_accountId(account_id2);
        println!("Let's debug get_cf_by_id: {:?}", cfs);
        let cfs = contract.get_inactive_cfs();
        println!("Let's debug inactive cfs: {:?}", cfs);
        let cfs = contract.get_active_accounts();
        println!("Let's debug active accounts: {:?}", cfs)
    }
    
}
