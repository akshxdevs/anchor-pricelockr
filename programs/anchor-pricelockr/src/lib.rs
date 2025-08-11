use anchor_lang::prelude::*;
use anchor_spl::token::{self,Token,Transfer};
use anchor_lang::solana_program::hash::{self};

declare_id!("ARhLmybiXeRiaERjEDWyTSqEri5JGt7xYvDLCtfqCFYg");

#[program]
pub mod anchor_pricelockr {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, nft:u64) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;
        let vault = &mut ctx.accounts.vault;        
        
        vault.owner = ctx.accounts.user.key();
        tournament.creator = ctx.accounts.creator.key();
        
        vault.bump = ctx.bumps.vault;
        tournament.bump = ctx.bumps.tournament;

        tournament.price_claimed = false;
        tournament.contestants = vec![];
        tournament.price_nft = nft;
        Ok(())
    }
    pub fn add_contestants(ctx: Context<AddContestants>,contestants:Vec<Pubkey>) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;
        for wallet in contestants.iter(){
            let contestant = Contestants { 
                id: tournament.contestants.len() as u64 + 1, 
                wallet_address: *wallet, 
            };
            tournament.contestants.push(contestant);
        };
        Ok(())
    }
    pub fn tournament_result(ctx: Context<TournamentResult>) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;
        let seed_data = [
            ctx.accounts.user.key().as_ref(),
        ].concat();
        let hash = hash::hash(&seed_data);
        let rn_index = (hash.to_bytes()[0] as usize) % tournament.contestants.len();
        let winner = tournament.contestants[rn_index].wallet_address;
        tournament.winner = winner;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            ctx.accounts.vault.owner.as_ref(),
            &[ctx.accounts.vault.bump],
        ]];   
        require!(tournament.winner != Pubkey::default(),CustomError::WinnerNotFound);

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.vault_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
         );
         token::transfer(cpi_ctx, tournament.price_nft)?;
        Ok(())
    }
    pub fn claim_reward(ctx: Context<ClaminPrice>) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;
        let winner = &mut ctx.accounts.winner;
        winner.bump =ctx.bumps.winner;
        require!(tournament.winner != Pubkey::default(), CustomError::WinnerNotFound);
        require!(!tournament.price_claimed, CustomError::AlreadyClaimed);
        require_keys_eq!(ctx.accounts.user.key(), tournament.winner, CustomError::WinnerNotFound);
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            ctx.accounts.vault.owner.as_ref(),
            &[ctx.accounts.vault.bump],
        ]];   
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_ata.to_account_info(),
                to: ctx.accounts.winner_ata.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
         );
         token::transfer(cpi_ctx, tournament.price_nft)?;
         tournament.price_claimed = true;
        Ok(())
    }
}

#[account]
pub struct Vault {
    pub owner:Pubkey,
    pub bump:u8,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Contestants{
    pub id:u64,
    pub wallet_address:Pubkey
}
#[account]
pub struct Tournament{
    pub creator:Pubkey,
    pub bump:u8,
    pub contestants: Vec<Contestants>,
    pub winner:Pubkey,
    pub price_claimed:bool,
    pub price_nft:u64,
}

#[account]
pub struct Winner{
    pub winner:Pubkey,
    pub bump:u8
}
#[derive(Accounts)]
pub struct Initialize <'info>{
    #[account(
        init,
        seeds = [b"nft",creator.key().as_ref()],
        payer = creator,
        space = 8 + 32 + 1 + 4 + (40*10) + 32 + 1 + 8,
        bump
    )]
    pub tournament:Account<'info,Tournament>,
    #[account(
        init,
        seeds = [b"vault",user.key().as_ref()],
        payer = user,
        space = 8 + 32 + 1,
        bump,
    )]
    pub vault: Account<'info,Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub creator:Signer<'info>,
    pub system_program:Program<'info,System>
}
#[derive(Accounts)]
pub struct AddContestants<'info>{
    #[account(mut)]
    pub tournament:Account<'info,Tournament>,
    #[account(mut)]
    pub creator:Signer<'info>,
    pub system_program:Program<'info,System>
}

#[derive(Accounts)]
pub struct TournamentResult<'info>{
    #[account(mut)]
    pub tournament:Account<'info,Tournament>,
    #[account(mut)]
    pub vault:Account<'info,Vault>,
    ///CHECKS vault token account
    #[account(mut)]
    pub vault_ata:AccountInfo<'info>,
    ///CHECKS user token account
    #[account(mut)]
    pub user_ata:AccountInfo<'info>,
    #[account(mut)]
    pub user:Signer<'info>,
    pub token_program:Program<'info,Token>,
    pub system_program:Program<'info,System>

}

#[derive(Accounts)]
pub struct ClaminPrice<'info>{
    #[account(mut)]
    pub tournament:Account<'info,Tournament>,
    #[account(mut)]
    pub vault:Account<'info,Vault>,
    #[account(
        mut,
        seeds = [b"win",user.key().as_ref()],
        bump
    )]
    pub winner:Account<'info,Winner>,
    ///CHECKS vault token account
    #[account(mut)]
    pub vault_ata:AccountInfo<'info>,
    ///CHECKS user token account
    #[account(mut)]
    pub user_ata:AccountInfo<'info>,
    ///CHECKS winner token account
    #[account(mut)]
    pub winner_ata:AccountInfo<'info>,
    #[account(mut)]
    pub user:Signer<'info>,
    pub token_program:Program<'info,Token>,
    pub system_program:Program<'info,System>

}
#[error_code]
pub enum CustomError {
    #[msg("Winner not founded!!")]
    WinnerNotFound,
    #[msg("Prize already claimed!")]
    AlreadyClaimed,
}
