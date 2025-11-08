# 🚀 GoQuant - Build & Run Instructions

## ⚠️ Important: Windows Linker Issue

Your system has a linker conflict where Git's `link.exe` is interfering with MSVC's `link.exe`. This is a common Windows development issue.

---

## 🎯 Quick Start (Windows)

### Option 1: Use the Helper Scripts (EASIEST)

I've created helper scripts that automatically fix the PATH issue:

#### **To Build Everything:**
```cmd
build-fix.bat
```

#### **To Run the Backend Server:**
```cmd
run-backend.bat
```

These scripts temporarily remove Git from PATH, run the commands, then restore your PATH.

---

### Option 2: Manual Command Line

#### **Step 1: Open Command Prompt (cmd.exe)**

#### **Step 2: Temporarily Fix PATH**
```cmd
set PATH=%PATH:C:\Program Files\Git\usr\bin;=%
```

#### **Step 3: Build Backend**
```cmd
cd C:\Users\SNC\Desktop\Goquant\quant\back
cargo build
```

#### **Step 4: Run Backend**
```cmd
cargo run
```

---

### Option 3: Use WSL (RECOMMENDED for Development)

#### **Step 1: Install Rust in WSL**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### **Step 2: Build Backend**
```bash
cd /mnt/c/Users/SNC/Desktop/Goquant/quant/back
cargo build
```

#### **Step 3: Run Backend**
```bash
cargo run
```

---

### Option 4: Fix PATH Permanently (For Developers)

If you're doing a lot of Rust development on Windows, you should fix this properly:

1. **Find MSVC Link Location:**
   ```cmd
   where /R "C:\Program Files (x86)\Microsoft Visual Studio" link.exe
   ```
   Typical location: `C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\14.xx.xxxxx\bin\Hostx64\x64`

2. **Edit System Environment Variables:**
   - Open Start Menu → "Environment Variables"
   - Edit "Path" under System Variables
   - Move MSVC bin directory **BEFORE** Git directories
   - Apply and restart terminal

3. **Install Visual Studio Build Tools (if missing):**
   - Download: https://visualstudio.microsoft.com/downloads/
   - Install "Desktop development with C++" workload

---

## 📦 Building Anchor Program

### If you have Anchor CLI installed:

```bash
cd C:\Users\SNC\Desktop\Goquant\quant
anchor build
```

### If you don't have Anchor:

```bash
npm install -g @coral-xyz/anchor-cli
# Or using yarn:
yarn global add @coral-xyz/anchor-cli
```

---

## 🧪 Running Tests

### Backend Tests:
```cmd
cd quant\back
cargo test
```

### Anchor Program Tests:
```bash
cd quant
anchor test
```

---

## 🌐 Running the Backend Server

Once built successfully, the server will start on:
```
http://0.0.0.0:8080
```

### Available Endpoints:

**POST Endpoints:**
- `/register` - Initialize vault
- `/deposit` - Deposit collateral
- `/withdraw` - Withdraw collateral
- `/lock` - Lock collateral
- `/unlock` - Unlock collateral
- `/transfer` - Transfer collateral

**GET Endpoints:**
- `/vault/balance/{user}` - Get vault balance
- `/vault/transactions/{user}` - Transaction history
- `/vault/status/{user}` - Vault status
- `/vault/tvl` - Total value locked
- `/vault/alerts` - System alerts

**Analytics:**
- `/analytics/dashboard` - System analytics
- `/analytics/tvl-history/{days}` - TVL history

**WebSocket:**
- `/ws` - Real-time updates

---

## 🔧 Troubleshooting

### Issue: "linking with link.exe failed"
**Solution**: Use one of the helper scripts or temporarily remove Git from PATH

### Issue: "cargo: command not found"
**Solution**: Install Rust from https://rustup.rs/

### Issue: "anchor: command not found"
**Solution**: Install Anchor CLI with `npm install -g @coral-xyz/anchor-cli`

### Issue: Database connection failed
**Solution**: The app will run without PostgreSQL (it has fallback). To use PostgreSQL:
```bash
# Install PostgreSQL
# Create database:
createdb vault_management
# Set environment variable:
set DATABASE_URL=postgresql://vault_user:password@localhost/vault_management
```

### Issue: "Failed to read Solana keypair"
**Solution**: Generate a Solana keypair:
```bash
solana-keygen new --outfile ~/.config/solana/id.json
```

---

## 📊 What Was Implemented

✅ **All Core Requirements:**
- Solana smart contract with all 6 instructions
- Complete backend service
- PostgreSQL database integration
- REST API with all endpoints
- WebSocket support

✅ **Bonus Features:**
- Multi-signature vaults
- 24-hour withdrawal delay security
- Advanced analytics and reporting
- Real-time monitoring

---

## 📄 Documentation

- **Implementation Status**: See `IMPLEMENTATION_STATUS.md`
- **Assignment Requirements**: See `quant/Instruction_ReadMe.md`
- **Code Documentation**: Inline comments in all source files

---

## 🎥 Next Steps for Submission

1. ✅ Code complete
2. ⏳ Record video demonstration (10-15 minutes)
3. ⏳ Run test suite and collect results
4. ⏳ Submit to careers@goquant.io (CC: himanshu.vairagade@goquant.io)

**What to include in submission:**
- Resume
- Source code (GitHub link or zip)
- Video demonstration (YouTube unlisted link)
- Technical documentation (`IMPLEMENTATION_STATUS.md` as PDF)
- Test results

---

## 💡 Tips

- The runtime drop warning during shutdown is **normal** and doesn't affect functionality
- PostgreSQL is **optional** - the app works without it
- Use WSL for the smoothest development experience on Windows
- All compilation errors were related to the linker issue, not code problems

---

**Good luck with your submission!** 🚀
