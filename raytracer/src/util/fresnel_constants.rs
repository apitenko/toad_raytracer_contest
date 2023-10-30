#[allow(non_upper_case_globals)]

// partially taken from http://hyperphysics.phy-astr.gsu.edu/hbase/Tables/indrf.html
pub mod FresnelConstants {
    pub const Vacuum: f32 = 1.00000;
    pub const Air: f32 = 1.00029; // at SFP
    pub const Ice: f32 = 1.31;
    pub const Water: f32 = 1.33; // at 20 C
    pub const Acetone: f32 = 1.36;
    pub const Ethanol: f32 = 1.36;
    pub const Sugar30: f32 = 1.38; // 30% solution
    pub const Fluorite: f32 = 1.433;
    pub const FusedQuartz: f32 = 1.46;
    pub const Glycerine: f32 = 1.473;
    pub const Sugar80: f32 = 1.49; // 80% solution
    pub const TypicalCrownGlass: f32 = 1.52;
    pub const CarbonDisulfide: f32 = 1.63;
    pub const HeavyFlintGlass: f32 = 1.65;
    pub const ExtraDenseFlint: f32 = 1.7200;
    pub const Sapphire: f32 = 1.77;
    pub const Diamond: f32 = 2.417;

    // ? our custom coefficients
    pub const ShaderToyExample: f32 = 1.125;
}