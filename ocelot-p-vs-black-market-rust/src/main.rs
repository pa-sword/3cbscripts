use std::{cmp, ops::Sub, vec};


const LOGGING: bool = true;
const TURN_LIMIT: u32 = 18;

/*
    In this simplified version, we start on the forced turn 4
    where ocelot attacked with two cats on turn 4A, against BMC's sole blocker

    the only relevant decision that BMC can take before that, is 
    1) whether to make a changeling
    (proving this is optimal is left as an exercise to the reader)

    2) and whether to make a treasure alongside it;
        I'm fairly certain lines where BMC doesn't make treasures at all have been explored.
        If  they don't start early, they lose access to the threat of lifelink, later anyways.
    ** in theory need to confirm this is optimal


    For the sake of speed and demonstrating ocelot advantage from here,
    ocelot player won't ever attack unless it has lethal.

*/

fn main() {
    let res = State {
        turn_count: 4
        , priority: Player::BMC
        , turn: Player::BMC
        , op_health: 24
        , bmc_health: 13
        , changeling: 1
        , changeling_tapped: 0
        , changeling_entered: 0
        , treasure: 2 // simulates the existence of peat bog while not having to specifically model it
        , cat: 2
        , cat_tapped: 1
        , ocelot_alive: 1
        , ocelot_tapped: 0
        , city_blessing: false 
        , legal_actions: vec![Action::MakeChangeling, Action::MakeTreasure, Action::MakeBoth],
    }.solve().to_string();
    
    // let res = State {
    //     turn_count: 15
    //     , priority: Player::BMC
    //     , turn: Player::BMC
    //     , op_health: 35
    //     , bmc_health: 2
    //     , changeling: 1
    //     , changeling_tapped: 0
    //     , changeling_entered: 0
    //     , treasure: 6 // simulates the existence of peat bog while not having to specifically model it
    //     , cat: 5
    //     , cat_tapped: 1
    //     , ocelot_alive: 1
    //     , ocelot_tapped: 0
    //     , city_blessing: false 
    //     , legal_actions: vec![Action::Pass],
    // }.solve().to_string();


    println!("{} wins the whole thing", res);
}

#[derive(Debug, Clone)]
struct State {
    turn_count: u32
    , priority: Player
    , turn: Player
    , op_health: i32
    , bmc_health: i32
    , changeling: u32
    , changeling_tapped: u32
    , changeling_entered: u32
    , treasure: u32
    , cat: u32
    , cat_tapped: u32
    , ocelot_alive: u32
    , ocelot_tapped: u32
    , city_blessing: bool
    , legal_actions: Vec<Action>
}

#[derive(Debug, Clone, Copy)]
enum Player {
    OP, BMC
}

impl Player {
    fn opponent(&self) -> Player {
        match self {
            Player::OP => Player::BMC,
            Player::BMC => Player::OP,
        }
    }

    fn equal(&self, other: Player) -> bool {
        match self {
            Player::OP => match other {
                Player::OP => true,
                Player::BMC => false,
            },
            Player::BMC => match other {
                Player::OP => false,
                Player::BMC => true,
            },
        }
    }

    fn to_string(&self) -> String {
        match self {
            Player::OP => "OP".to_string(),
            Player::BMC => "BMC".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    MakeChangeling, MakeTreasure, MakeBoth, AutoBlock,
    Pass, Attack, Block, Vault,
}

fn make_changeling( state: &State ) -> State {
    if LOGGING { print!("make_changeling, ") };
    let mut ret = state.clone();
    ret.bmc_health = state.bmc_health - 3;
    ret.changeling = state.changeling + 1;
    ret.changeling_entered = 1;
    if state.changeling > 0 {
        ret.legal_actions = vec![Action::Attack, Action::Pass];
    } else {
        ret.legal_actions = vec![Action::Pass];
    }
    ret
}

fn make_treasure( state: &State ) -> State {
    if LOGGING { print!("make_treasure, ") };
    let mut ret = state.clone();
    ret.bmc_health = state.bmc_health - 1;
    ret.treasure = state.treasure + 1;
    if state.changeling > 0 {
        ret.legal_actions = vec![Action::Attack, Action::Pass];
    } else {
        ret.legal_actions = vec![Action::Pass];
    }
    ret
}

fn make_both( state: &State ) -> State {
    if LOGGING { print!("make_both, ") };
    let mut ret = state.clone();
    ret.bmc_health = state.bmc_health - 4;
    ret.changeling = state.changeling + 1;
    ret.changeling_entered = state.changeling_entered + 1;
    ret.treasure = state.treasure + 1;
    if state.changeling > 0 {
        ret.legal_actions = vec![Action::Attack, Action::Pass];
    } else {
        ret.legal_actions = vec![Action::Pass];
    }
    ret
}

fn vault( state: &State ) -> State {
    if LOGGING { print!("vault activation") };
    let mut ret = state.clone();
    ret.treasure -= 4u32;

    match state.turn {
        Player::OP => {
            let blockers = state.changeling - state.changeling_tapped;
            ret.bmc_health = state.bmc_health.checked_add_unsigned( 3 * blockers ).unwrap();
            ret.priority = Player::BMC;
            ret.legal_actions = vec![Action::AutoBlock];

            ret
        },
        Player::BMC => {
            ret.bmc_health = state.bmc_health.checked_add_unsigned( 3 * state.changeling_tapped ).unwrap();
            ret.priority = Player::OP;
            ret.legal_actions = vec![Action::Block];
            ret
            
        },
    }
}

fn pass( state: &State ) -> State {
    if LOGGING {
        println!();
        state.print();
    }

    let mut ret = state.clone();

   match state.priority {
    Player::OP => {
        if state.ocelot_alive == 1  {
            ret.cat = state.cat + 1;
            if ret.cat >= 7 {
                ret.city_blessing = true;
            }

            if ret.city_blessing {
                ret.cat += 1;
            }
        }
        ret.changeling_tapped = 0;
        ret.priority = Player::BMC;
        ret.turn = Player::BMC;
        ret.legal_actions = vec![Action::MakeChangeling, Action::MakeTreasure, Action::MakeBoth];

        ret
    },
    Player::BMC => {
        ret.changeling_entered = 0;
        ret.cat_tapped = 0;
        ret.ocelot_tapped = 0;
        ret.op_health = state.op_health + 1;
        ret.priority = Player::OP;
        ret.turn_count = state.turn_count + 1;
        ret.turn = Player::OP;
        if state.cat >= state.changeling - state.changeling_tapped {
            ret.legal_actions = vec![Action::Attack, Action::Pass];
        } else {
            ret.legal_actions = vec![Action::Pass];
        }

        ret
    },
   }
}

fn attacks( state: &State ) -> Vec<State> {
    if LOGGING { print!("attacks, ") };
   let mut ret: Vec<State> = vec![];

   /* 
   There is only all out attack entertained as a viable attack for OP (outside from PASS) 

   Note BMC still has every possible number of attackers tried out
   */
   match state.priority {
    Player::OP => {
        let mut all_out_attack = state.clone();
        all_out_attack.cat_tapped = state.cat;
        all_out_attack.priority = Player::BMC;
        let mut all_out_ocelot = all_out_attack.clone();
        all_out_ocelot.ocelot_tapped = 1;


        if state.treasure >= 4 {    
            all_out_ocelot.legal_actions = vec![Action::AutoBlock, Action::Vault];
            all_out_attack.legal_actions = vec![Action::AutoBlock, Action::Vault];

        } else {
            all_out_ocelot.legal_actions = vec![Action::AutoBlock];
            all_out_attack.legal_actions = vec![Action::AutoBlock];
        }
        ret.push(all_out_attack);
        ret.push(all_out_ocelot);

        ret
    },
    Player::BMC => {
        let mut ret: Vec<State> = vec![];
        let max_attackers = state.changeling - state.changeling_entered;
        for attackers in 1..max_attackers {
            let mut attack = state.clone();
            attack.changeling_tapped = attackers;
            attack.priority = Player::OP;
            attack.legal_actions = vec![Action::Block];

            ret.push( attack );

            if state.treasure >= 4 { 
                let mut attack_vault = state.clone();
                attack_vault.changeling_tapped = attackers;
                attack_vault.legal_actions = vec![Action::Vault];

                ret.push( attack_vault );
            }
        }
        ret
    },
   }
}

fn block( state: &State ) -> State {
    if LOGGING { print!("block, ") };
   let mut ret = state.clone();

   match state.priority {
    Player::OP => {
        panic!("block is always a multi-option case for OP player, use blocks")
    },
    // Ocelot will never attack unless it wins
    Player::BMC => {
        let mut blockers_to_apply =  state.changeling - state.changeling_tapped; 
        // let damage: u32 = state.cat_tapped + state.ocelot_tapped - blockers_to_apply;
        let damage: u32 = state.cat_tapped - blockers_to_apply;
        ret.bmc_health = state.bmc_health.checked_sub_unsigned(damage).unwrap();
    
        // if state.ocelot_tapped == 1u32 {
        //     ret.op_health = state.op_health + 1;
        //     if blockers_to_apply > 0 {
        //         ret.ocelot_alive = 0;
        //         blockers_to_apply -= 1;
        //     }
        // }
        ret.cat = state.cat - blockers_to_apply; 
        ret.cat_tapped = state.cat_tapped - blockers_to_apply;
        ret.priority = Player::OP;
        ret.legal_actions = vec![Action::Pass];

        ret
    },
   }
}

// unless ocelot wins on crackback, it trades with as many cats as possible

fn blocks( state: &State) -> Vec<State> {
    if LOGGING { print!("blocks, ") };
    let mut ret: Vec<State> = vec![];

    match state.priority {
        Player::OP => {

            // Break Scenario:
            // If there are enough attackers to win on the backswing,
            // we return a winning ocelot state (shortcutting calcs)

            let necessary_blocks_signed: i32 = (-state.op_health / 3 ).checked_add_unsigned(state.changeling_tapped).unwrap() ;
            let necessary_blocks: u32 = if necessary_blocks_signed.is_negative() { 0u32 } else { necessary_blocks_signed.unsigned_abs() };
            
            let potential_next_attackers = (state.cat + state.ocelot_alive).checked_sub(necessary_blocks).or( Some(0 ) ).unwrap() ;
            let potential_next_blockers = state.changeling - state.changeling_tapped;

            // NOTE: any Vault activation is done before blocks
            // but it doesn't hurt to be safe
            if potential_next_attackers > potential_next_blockers 
            && state.bmc_health.checked_sub_unsigned(potential_next_attackers - potential_next_blockers).unwrap() <= 0
            && state.treasure < 4  {
                ret = vec![State{ turn_count: 0, priority: Player::BMC, turn: Player::BMC, op_health: 20, bmc_health: -5, changeling: 0, changeling_tapped: 0, changeling_entered: 0, treasure: 0, cat: 1, cat_tapped: 1, ocelot_alive: 1, ocelot_tapped: 1, city_blessing: false, legal_actions: vec![] }];
                return ret
            }

            // case trade // ASSUMPTION: if trading is optimal, trading to the maximum amount is optimal (might be wrong)
            let max_blocker_groups = ( state.cat - state.cat_tapped ) / 2;
            let max_blocks = cmp::min( max_blocker_groups, state.changeling_tapped);

            let mut trade = state.clone();
            trade.changeling = state.changeling - max_blocks;
            trade.changeling_tapped = state.changeling_tapped - max_blocks;
            trade.cat = state.cat - ( max_blocks * 2 );
            trade.op_health = state.op_health.checked_sub_unsigned( 3 * trade.changeling_tapped ).unwrap();
            trade.priority = Player::BMC;
            trade.legal_actions = vec![Action::Pass];

            ret.push( trade );

            ret
        },
        Player::BMC => {
            panic!("blocks are deterministic for BMC player, use block")
        },
    }

}

impl State {
    fn solve( &self ) -> Player {
       // self.print();

        if self.op_health <= 0 {
            if LOGGING { print!("BMC wins") };
            return Player::BMC
        }

        if self.bmc_health <= 0 {
            if LOGGING { print!("OP wins") };
            return Player::OP
        }

        // turns out I need a stop condition in case i go down the "always pass" route
        // ocelot shouldn't attack, except in the win shortcut for crackback win
        if self.ocelot_alive == 0 && self.changeling > self.cat {
            panic!("ocelot should not be attacking ever in this simplified version of calcs");
        }

        // arbitrary turn limit to avoid pass/pass lines
        // we give BMC the edge, just to make sure ocelot does force the win
        if self.turn_count > TURN_LIMIT {
            if LOGGING { print!("turn limit was hit, ") };
            return Player::BMC
        }

// PLEASE CHECK THIS TO DEATH
// basically: if the player making the decision ("player with priority")
// finds a winning line from a set gamestate in their list of options,
// they stop looking for other options
// 
// if they find a loss, they look into other options until they find a win
// if no possible win is found from the initial gamestate,
// they "admit" this gamestate is lost to them



        <Vec<Action> as Clone>::clone(&self.legal_actions).into_iter()
            .fold( self.priority.opponent(), |winner_in_current_line, action| {
                if winner_in_current_line.equal( self.priority ) {
                    return self.priority
                }
                return match action {
                    Action::MakeChangeling => make_changeling(self).solve(),
                    Action::MakeTreasure => make_treasure(self).solve(),
                    Action::MakeBoth => make_both(self).solve(),
                    Action::Pass => pass(self).solve(),
                    Action::Attack => attacks( self ).into_iter()
                        .fold( self.priority.opponent(),|winner_in_current_line, new_line_of_play| {
                            if winner_in_current_line.equal( self.priority ) {
                                return self.priority
                            }
                            return new_line_of_play.solve()

                        }),
                    Action::AutoBlock => block(self).solve(),
                    Action::Block => blocks(self).into_iter()
                        .fold( self.priority.opponent(),|winner_in_current_line, new_line_of_play| {
                            if winner_in_current_line.equal( self.priority ) {
                                return self.priority
                            }
                            return new_line_of_play.solve()

                        }),
                    Action::Vault => vault(self).solve()
                }
            } 
        )

    }

    fn print(&self) {
        let padding = (self.turn_count * 2) as usize;
        println!("{:padding$}{}: {}-{}; {} changelings, {} treasures, {} cats, {} ocelot "
            , self.turn_count
            , self.turn.to_string()
            , self.op_health
            , self.bmc_health
            , self.changeling
            , self.treasure
            , self.cat
            , self.ocelot_alive
        )
    }

    fn compare(&self, other: State ) -> Option<Player> {
        if self.op_health >= other.op_health
        && self.bmc_health <= other.op_health
        && ( 
            self.priority.equal(Player::OP) || other.priority.equal( Player::BMC ) 
        )
        && self.cat >= other.cat
        && ( self.ocelot_alive == 1 || self.ocelot_alive == 0 )
        && self.changeling <= other.changeling
        {
            // the current state is strictly more favorable to OP
            Some(Player::OP)
        }
    }
}