#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod nft_contract {
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    #[derive(scale::Decode, scale::Encode, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct NFT {
        username: String,
        item: String,
    }

    #[ink(storage)]
    pub struct NftContract {
        nfts: Mapping<u32, NFT>,
        owner: Mapping<u32, AccountId>,
        next_token_id: u32,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        token_id: u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TokenNotFound,
        NotTokenOwner,
        TokenAlreadyExists,
        TokenIdOverflow,
    }

    impl NftContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                nfts: Mapping::default(),
                owner: Mapping::default(),
                next_token_id: 0,
            }
        }

        #[ink(message)]
        pub fn mint(&mut self, username: String, item: String) -> Result<u32, Error> {
            let token_id = self.next_token_id;
            let caller = self.env().caller();

            if self.nfts.contains(token_id) {
                return Err(Error::TokenAlreadyExists);
            }

            let nft = NFT { username, item };
            self.nfts.insert(token_id, &nft);
            self.owner.insert(token_id, &caller);
            self.next_token_id = self.next_token_id.checked_add(1).ok_or(Error::TokenIdOverflow)?;

            self.env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                token_id,
            });

            Ok(token_id)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, token_id: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            
            if !self.owner.contains(token_id) {
                return Err(Error::TokenNotFound);
            }

            if self.owner.get(token_id).unwrap() != caller {
                return Err(Error::NotTokenOwner);
            }

            self.owner.insert(token_id, &to);

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(to),
                token_id,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn get_nft(&self, token_id: u32) -> Option<NFT> {
            self.nfts.get(token_id)
        }

        #[ink(message)]
        pub fn update_username(&mut self, token_id: u32, new_username: String) -> Result<(), Error> {
            let caller = self.env().caller();
            
            if !self.owner.contains(token_id) {
                return Err(Error::TokenNotFound);
            }

            if self.owner.get(token_id).unwrap() != caller {
                return Err(Error::NotTokenOwner);
            }

            let mut nft = self.nfts.get(token_id).unwrap();
            nft.username = new_username;
            self.nfts.insert(token_id, &nft);

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        #[ink::test]
        fn mint_works() {
            let mut contract = NftContract::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            let username = String::from("Alice");
            let item = String::from("Sword");

            assert_eq!(contract.mint(username.clone(), item.clone()), Ok(0));

            let nft = contract.get_nft(0).unwrap();
            assert_eq!(nft.username, username);
            assert_eq!(nft.item, item);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = NftContract::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            let token_id = contract.mint(String::from("Alice"), String::from("Sword")).unwrap();

            assert_eq!(contract.transfer(accounts.bob, token_id), Ok(()));

            // Try to transfer again (should fail)
            assert_eq!(contract.transfer(accounts.charlie, token_id), Err(Error::NotTokenOwner));
        }

        #[ink::test]
        fn update_username_works() {
            let mut contract = NftContract::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            let token_id = contract.mint(String::from("Alice"), String::from("Sword")).unwrap();

            let new_username = String::from("AliceNewName");
            assert_eq!(contract.update_username(token_id, new_username.clone()), Ok(()));

            let nft = contract.get_nft(token_id).unwrap();
            assert_eq!(nft.username, new_username);
        }

        #[ink::test]
        fn cannot_update_item() {
            let mut contract = NftContract::new();
            let accounts = test::default_accounts::<DefaultEnvironment>();

            let username = String::from("Alice");
            let item = String::from("Sword");

            let token_id = contract.mint(username.clone(), item.clone()).unwrap();

            // Try to update the item (not possible in this implementation)
            let nft = contract.get_nft(token_id).unwrap();
            assert_eq!(nft.item, item);
        }
    }
}